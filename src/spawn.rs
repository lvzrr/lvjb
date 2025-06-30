use crate::config::*;
use crate::fs::*;
use std::process::*;
use std::path::PathBuf;

pub const ORANGE: &str = "\x1b[33m";
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const RESET: &str = "\x1b[0m";

pub fn  spawn_compilation_command(files: &Vec<PathBuf>, config: &Config) -> Result<(), Box<dyn std::error::Error>>
{
    run_hooks(&config.pre_build_cmds)?;
    if files.is_empty()
    {
        eprintln!("{GREEN}[COMPILER]{RESET} Nothing to compile");
        return Ok(());
    }
    let mut command = Command::new(&config.compiler);
    let classpath = expand_classpath(&config.classpath);
    if !config.classpath.is_empty()
    {
        command.arg("-cp").arg(&classpath);
    }
    command.arg("-d").arg(&config.paths.bin);
    for file in files
    {
        command.arg(file);
    }
    if let Some(x) = &config.args.compilation
    {
        command.args(x);
    }
    println!("{ORANGE}[COMPILER]{RESET} classpath: {}, output to: {}", &classpath, &config.paths.bin);
    let total = files.len();
    for (i, file) in files.iter().enumerate()
    {
        let symbol = if i < total - 1 { "├ " } else { "└ " };
        println!("  {} {}", symbol, file.to_string_lossy());
    }
    match command.status()
    {
        Ok(status) if status.success() => println!("{GREEN}[COMPILER OK]{RESET} Compilation succeeded."),
        Ok(status) => { return Err(format!("{RED}[COMPILER ERROR]{RESET} Compilation failed with status: {status}").into());},
        Err(err) => { return Err(format!("{RED}[COMPILER ERROR]{RESET} Failed to execute command: {err}").into()); },
    }
    run_hooks(&config.post_build_cmds)?;
    Ok(())
}

#[inline(always)]
pub fn  run_hooks(hooks: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for s in hooks {
        eprintln!("{ORANGE}[PRECOMP HOOK]{RESET} Running {}", s);
        let status = Command::new("sh").arg("-c").arg(&s).status();
        match status
        {
            Ok(code) if code.success() => continue,
            Ok(code) =>
            {
                return Err(format!("{RED}[HOOK ERROR]{RESET} `{}` failed with status: {}", s, code).into());
            }
            Err(err) =>
            {
                return Err(format!("{RED}[HOOK ERROR]{RESET} `{}` failed to execute: {err}", s).into());
            }
        }
    }
    Ok(())
}
