use std::{fs, fs::write};
use std::io::{stderr, copy, Write};
use std::path::PathBuf;
use crate::config::*;
use crate::fs::*;
use crate::cache::*;
use crate::incremental::*;
use crate::spawn::*;
use crate::jvm::*;
use std::thread;
use jni::objects::*;
use jni::*;
use std::process::Command;
use std::sync::{Arc, Mutex};
use reqwest::blocking::get;

pub const ORANGE: &str = "\x1b[33m";
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const RESET: &str = "\x1b[0m";

#[inline(always)]
pub fn  init(config: &Config) -> Result<(), Box<dyn std::error::Error>>
{
    let cache: Cache = Cache::default();
    if fs::metadata(CONF_FILE).is_err()
    {
        config.write()?;
    }
    if fs::metadata(CACHE_FILE).is_err()
    {
        cache.write()?;
    }
    fs::create_dir_all(&config.paths.src)?;
    fs::create_dir_all(&config.paths.bin)?;
    fs::create_dir_all(PathBuf::from(&config.paths.src).join(&config.paths.src_nopkg))?;
    fs::create_dir_all(&config.paths.test)?;
    fs::create_dir_all(&config.paths.lib)?;
    fs::create_dir_all(&config.paths.docs)?;
    fs::create_dir_all(&config.paths.releases)?;
    Ok(())
}

#[inline(always)]
pub fn  initpkg(mut s: String, config: &Config) -> Result<(), Box<dyn std::error::Error>>
{
    let p: PathBuf = forge_sys_path(&class_to_path(&mut s), config, PathType::SRC);
    fs::create_dir_all(&p)?;
    Ok(())
}

pub fn build(pkg: Option<&String>, config: &mut Config) -> Result<(), Box<dyn std::error::Error>>
{
    let mut f: bool = false;
    let pkpath = match pkg
    {
        Some(name) =>
        {
            if name.as_str() == "all" || config.incremental == false
            {
                PathBuf::from(&config.paths.src)
            }
            else
            {
                f = true;
                forge_sys_path(&class_to_path(name), config, PathType::SRC)
            }
        },
        None => 
        {
            forge_sys_path(&config.paths.src_nopkg, config, PathType::SRC)
        },
    };
    let mut files: Vec<PathBuf> = match config.incremental
    {
        true => fetch_files_under(&pkpath, &config.src_ext)
            .into_iter()
            .filter(|x| check_incremental(x, config))
            .collect(),
        false => fetch_files_under(&pkpath, &config.src_ext),
    };
    if f
    {
        let default_path = forge_sys_path(&config.paths.src_nopkg, config, PathType::SRC);
        let default_files = fetch_files_under(&default_path, &config.src_ext)
            .into_iter()
            .filter(|x| check_incremental(x, config));
        files.extend(default_files);
    }

    spawn_compilation_command(&files, config)?;

    if let Err(e) = config.cache.write()
    {
        eprintln!("{RED}[COMPILER]{RESET}Error saving cache: {e}");
    }
    Ok(())
}


pub fn run(p: Option<&String>, config: &Config, jvm: &JavaVM, output: bool) -> Result<(), Box<dyn std::error::Error>>
{
    let pkg = &match p
    {
        Some(s) => s,
        None => &match &config.entry_point
        {
            Some(x) => x.clone(),
            None =>
            {
                return Err(format!("{RED}[RUNNER]{RESET} No entry point").into());
            }
        },
    };

    let mut env = jvm.attach_current_thread()?;

    let class_name = pkg.replace(".", "/");
    let class = env.find_class(&class_name)?;

    let empty_args = Vec::new();
    let args_list = config.args.runtime.as_ref().unwrap_or(&empty_args);

    let string_class = env.find_class("java/lang/String")?;
    let args_array = env.new_object_array(args_list.len() as i32, string_class, JObject::null())?;

    for (i, arg) in args_list.iter().enumerate()
    {
        let jstr = env.new_string(arg)?;
        env.set_object_array_element(&args_array, i as i32, jstr)?;
    }

    let result = env.call_static_method(
        class,
        "main",
        "([Ljava/lang/String;)V",
        &[JValue::Object(&JObject::from(args_array))],
    );

    if let Err(jni::errors::Error::JavaException) = result
    {
        if env.exception_check()?
        {
            let exception = env.exception_occurred()?;
            env.exception_clear()?;

            let jstr = env.call_method(exception, "toString", "()Ljava/lang/String;", &[])?;
            let msg_obj = jstr.l()?;
            let msg: String = env.get_string(&JString::from(msg_obj))?.into();

            return Err(format!("{RED}[EXCEPTION]{RESET} {msg}").into());
        }
        return Err("[RUNNER] Java exception occurred, but couldn't get details".into());
    }
    else
    {
        result?;
    }
    if output
    {
        eprintln!("{GREEN}[RUNNER OK]{RESET}");
    }
    Ok(())
}

pub fn test(config: &mut Config) -> Result<(), Box<dyn std::error::Error>>
{
    let pkpath = PathBuf::from(&config.paths.test);

    let files: Vec<PathBuf> = if config.incremental
    {
        fetch_files_under(&pkpath, &config.src_ext)
            .into_iter()
            .filter(|x| check_incremental(x, config))
            .collect()
    }
    else
    {
        fetch_files_under(&pkpath, &config.src_ext)
    };

    spawn_compilation_command(&files, config)?;

    let all = fetch_files_under(&pkpath, &config.src_ext);

    config.cache.write()?;

    let jvm = spawn_jvm(config)?;
    let jvm = std::sync::Arc::new(jvm);

    let ok_tests: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::with_capacity(all.len())));
    let allowed_n = thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    for chunk in all.chunks(allowed_n)
    {
        let mut handles = Vec::new();
        for file in chunk.to_vec()
        {
            let src = config.paths.test.clone();
            let bin = config.paths.bin.clone();
            let ext = config.src_ext.clone();
            let conf = config.clone();
            let jvmp = Arc::clone(&jvm);
            let ok_vec = Arc::clone(&ok_tests);
            handles.push(thread::spawn(move ||
            {
                if let Ok(relative) = file.strip_prefix(&src)
                {
                    let mut new_path = PathBuf::from(&bin);
                    new_path.push(relative);
                    if new_path.extension().map(|e| *e == *ext).unwrap_or(false)
                    {
                        new_path.set_extension("");
                    }
                    let class_name = new_path.strip_prefix(&bin)
                        .unwrap_or(&new_path)
                        .with_extension("")
                        .to_string_lossy()
                        .replace("/", ".")
                        .replace("\\", ".");
                    if let Err(e) = run(Some(&class_name), &conf, &jvmp, false)
                    {
                        eprintln!("{RED}[TEST FAILED]{RESET} {}: {e}", class_name);
                        let _ = stderr().flush();
                    }
                    else
                    {
                        ok_vec.lock()
                        .expect(&format!("{RED}[TESTRUNNER]{RESET} Failed to lock test result vector: {}", std::io::Error::last_os_error()))
                        .push(class_name);
                        let _ = stderr().flush();
                    }
                }
            }));
        }
        for handle in handles {
            match handle.join()
            {
                Ok(_) => (),
                Err(_) => return Err(format!("{RED}[TESTRUNNER]{RESET} Failed to join handles").into()),
            }
        }
    }

    let passed = match ok_tests.lock()
    {
        Ok(x) => x,
        Err(e) => return Err(format!("{RED}[TESTRUNNER]{RESET} Failed to lock test result vector: {}", e).into()),
    };
    if passed.is_empty()
    {
        eprintln!("{RED}[TESTRUNNER]{RESET} No tests passed.");
    }
    else
    {
        eprint!("{GREEN}[PASSED TESTS]{RESET}: ");
        for ok in passed.iter()
        {
            eprint!("{} ", ok);
    }
        eprintln!();
    }
    Ok(())
}

#[inline(always)]
pub fn  clean(config: &Config) -> Result<(), Box<dyn std::error::Error>>
{
    let files = fetch_files_under(&PathBuf::from(&config.paths.bin), &"class".to_string());
    for file in files
    {
        fs::remove_file(file)?;
    }
    fs::remove_file(CACHE_FILE)?;
    Ok(())
}

pub fn  docgen(s: &str, config: &Config)
{
    let classpath = expand_classpath(&config.classpath);
    let src_path = forge_sys_path(s, config, PathType::SRC);
    let files = fetch_files_under(&src_path, &config.src_ext);

    let mut cmd = Command::new("javadoc");
    cmd.arg("-d").arg(&config.paths.docs);
    cmd.arg("-cp").arg(&classpath);
    cmd.args(&files);

    if let Err(e) = cmd.status()
    {
        eprintln!("{RED}[DOCGEN]{RESET} Failed to run javadoc: {}", e);
    }
}

#[inline(always)]
pub fn curl(url: &String, config: &mut Config) -> Result<(), Box<dyn std::error::Error>>
{
    eprintln!("{ORANGE}[FETCHING]{RESET} {}", url);
    let response = get(url)?;
    let filename = url.rsplit('/').next().ok_or_else(||
    {
        format!("{RED}[FETCHER]{RESET} Invalid URL")
    })?;
    let dest_path: PathBuf = forge_sys_path(filename, config, PathType::LIB);
    let mut dest = fs::File::create(dest_path)?;
    let mut content = response;
    copy(&mut content, &mut dest)?;
    eprintln!("{GREEN}[FETCHED]{RESET} {}", filename);
    config.cache.url_libs.push(url.to_owned());
    config.cache.write()?;
    Ok(())
}


pub fn  release(config: &mut Config) -> Result<(), Box<dyn std::error::Error>> {
    build(Some(&"all".to_string()), config)?;
    let main_class = match &config.entry_point
    {
        Some(main) => main,
        None => return Err(format!("{RED}[RELEASE]{RESET} No entry point set in config").into()),
    };
    let manifest_content = format!("Main-Class: {}\n", main_class);
    let manifest_path = PathBuf::from("MANIFEST.MF");
    write(&manifest_path, manifest_content)?;
    let out = format!("{}-{}.jar", &config.jar, &config.version);
    let jar_path = forge_sys_path(&out, &config, PathType::RELEASES);
    let status = Command::new("jar")
        .arg("cfm")
        .arg(&jar_path)
        .arg(&manifest_path)
        .arg("-C")
        .arg(&config.paths.bin)
        .arg(".")
        .status()?;

    if !status.success()
    {
        return Err(format!("{RED}[RELEASE]{RED} jar command failed").into());
    }
    eprintln!("{GREEN}[RELEASE]{RESET} Created: {}", jar_path.display());
    fs::remove_file("MANIFEST.MF")?;
    config.cache.releases.push(
       Release  {
                    cnf: Some(config.clone()),
                    jar: Some(config.jar.clone()),
                }
    ); 
    config.cache.write()?;
    Ok(())
}

pub fn  help()
{
    println!("{GREEN}jmake - fast, minimal Java build + test tool{RESET}");
    println!();
    println!("{ORANGE}Usage:{RESET}");
    println!("  jmk <command> [args]");
    println!();
    println!("{ORANGE}Available Commands:{RESET}");
    println!("  init                       Initializes project structure and config");
    println!("  initpkg <pkg>              Creates folder tree under src/ for given package");
    println!("  build [pkg|all] [--re]     Builds Java sources (incrementally unless --re)");
    println!("  test                       Compiles and runs test files (via JNI, parallel)");
    println!("  run [MainClass]            Runs specified Java class or entry_point from config");
    println!("  clean                      Deletes all .class files and clears cache");
    println!("  docgen <Class>             Generates Javadoc for specified class");
    println!("  curl <url>                 Downloads and registers remote JAR");
    println!("  release                    Builds JAR from entry_point and config values");
    println!("  help                       Displays this help message");
    println!();
    println!("{ORANGE}Quirks & Notes:{RESET}");
    println!("  - Always compiles default/ (no-package) sources, even if building a package.");
    println!("  - test/ files are treated as standalone Java programs, no framework needed.");
    println!("  - Classpath expansion supports wildcards like lib/*");
    println!("  - Incremental builds use fast xxh3 hashing (not timestamps).");
    println!("  - Remote JARs via 'curl' are cached and reused.");
    println!("  - Release creates a JAR using 'jar' tool and Main-Class from config.");
    println!("  - If jmk.toml or jmk.lock doesn't exist, they’re auto-generated.");
    println!();
    println!("{ORANGE}Example:{RESET}");
    println!("  jmk init");
    println!("  jmk initpkg com.example.app");
    println!("  jmk build com.example.app");
    println!("  jmk run com.example.app.Main");
    println!();
}
