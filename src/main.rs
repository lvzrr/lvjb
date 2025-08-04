use lvjb::config::*;
use lvjb::{cmds, cmds::*};
use lvjb::jvm::*;
use std::env;

fn  _main() -> Result<(), i32> {
    let args: Vec<String> = env::args().collect();
    let mut conf = match Config::load()
    {
        Ok(x) => x,
        Err(_) =>
            match args.get(1)
            {
                Some(x) if x == &"--help".to_string() =>
                {
                    cmds::help();
                    return Ok(());
                },
                Some(x) if x != &"init".to_string() =>
                {
                    eprintln!("{RED}[lvjb]{RESET} not a lvjb directory, run 'lvjb init' to initialize it, or run --help for more info");
                    return Err(1);
                },
                Some(x) if x == &"init".to_string() => Config::default(),
                Some(_) =>
                {
                    eprintln!("{RED}[lvjb]{RESET} not a lvjb directory, run 'lvjb init' to initialize it, or run --help for more info");
                    return Err(1);
                }
                None =>
                {
                    eprintln!("{RED}[lvjb]{RESET} not a lvjb directory, run 'lvjb init' to initialize it, or run --help for more info");
                    return Err(1);
                }
            },
    };
    match args.get(1).map(String::as_str)
    {
        Some("init") =>
        {
            if let Err(e) = cmds::init(&conf)
            {
                eprintln!("{RED}[lvjb]{RESET} {e}");
                return Err(1);
            }
        }
        Some("initpkg") =>
        {
            if let Some(pkg) = args.get(2)
            {
                if let Err(e) = cmds::initpkg(pkg.clone(), &conf)
                {
                    eprintln!("{RED}[lvjb]{RESET} {e}");
                    return Err(1);
                }
            }
            else
            {
                eprintln!("{RED}[lvjb]{RESET} Missing package name for 'initpkg'");
                return Err(1);
            }
        }
        Some("build") =>
        {
            if args.contains(&"--re".to_string())
            {
                conf.incremental = false;
            }
            let pkg = args.get(2);
            if let Err(e) = cmds::build(pkg.clone(), &mut conf)
            {
                eprintln!("{e}");
                return Err(1);
            }
        }
        Some("docgen") =>
        {
            if let Some(classname) = args.get(2)
            {
                cmds::docgen(classname, &mut conf)
            }
            else
            {
                eprintln!("{RED}[DOCGEN ERROR]{RESET} No class specified");
                return Err(1);
            }
        }
        Some("curl") =>
        {
            if let Some(url) = args.get(2)
            {
                if let Err(e) = cmds::curl(url, &mut conf)
                {
                    eprintln!("{RED}[CURL ERROR]{RESET} {e}");
                    return Err(1);
                }
            }
            else
            {
                eprintln!("{RED}[CURL ERROR]{RESET} No url specified");
                return Err(1);
            }
        }
        Some("run") => {
            let extra_args_start = args.iter().position(|arg| arg == "--");

            if let Ok(jvm) = spawn_jvm(&conf)
            {
                let pkg = match args.get(2)
                {
                    Some(x) if x == "--" => match conf.entry_point
                    {
                        Some(ref s) => Some(s.to_string()),
                        None => None,
                    },
                    Some(s) => Some(s.to_string()),
                    None => None,
                };
                if let Some(pos) = extra_args_start
                {
                    let user_args = args[pos + 1..].to_vec();
                    conf.args.runtime.get_or_insert_with(Vec::new).extend(user_args);
                }

                if let Err(e) = cmds::run(pkg.as_ref(), &mut conf, &jvm, true)
                {
                    eprintln!("{e}");
                    return Err(1);
                }
            }
            else if let Err(e) = spawn_jvm(&conf)
            {
                eprintln!("{e}");
                return Err(1);
            }
        }
        Some("test") =>
        {
            if let Err(e) = cmds::test(&mut conf)
            {
                eprintln!("{e}");
                return Err(1);
            }
        }
        Some("clean") =>
        {
            if let Err(e) = cmds::clean(&conf)
            {
                eprintln!("{e}");
                return Err(1);
            }
        }
        Some("release") =>
        {
            if let Err(e) = cmds::release(&mut conf)
            {
                eprintln!("{e}");
                return Err(1);
            }
        }
        Some(cmd) =>
        {
            eprintln!("{RED}[lvjb]{RESET} Unrecognized command: '{}'", cmd);
            return Err(1);
        }
        None =>
        {
            eprintln!("{RED}[lvjb]{RESET} No command provided");
            return Err(1);
        }
    }
    Ok(())
}

fn main() {
    std::process::exit(match _main() {
        Ok(_) => 0,
        Err(code) => code,
    });
}
