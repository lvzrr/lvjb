use crate::config::*;
use std::path::PathBuf;
use std::fs;

pub enum PathType
{
    SRC,
    SRCNOPKG,
    BIN,
    LIB,
    TEST,
    DOCS,
    RELEASES,
}

#[inline(always)]
pub fn  forge_sys_path(path: &str, config: &Config, ptype: PathType) -> PathBuf
{
    let mut out = match ptype
    {
        PathType::SRC       => PathBuf::from(&config.paths.src),
        PathType::SRCNOPKG  => PathBuf::from(&config.paths.src_nopkg),
        PathType::BIN       => PathBuf::from(&config.paths.bin),
        PathType::LIB       => PathBuf::from(&config.paths.lib),
        PathType::TEST      => PathBuf::from(&config.paths.test),
        PathType::DOCS      => PathBuf::from(&config.paths.docs),
        PathType::RELEASES  => PathBuf::from(&config.paths.releases),
    };
    out.push(path);
    out
}

#[inline(always)]
pub fn  class_to_path(s: &str) -> String
{
    s.replace(".", "/")
}

#[inline(always)]
pub fn fetch_files_under(p: &PathBuf, src_ext: &String) -> Vec<PathBuf>
{
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(p)
    {
        for entry in entries.flatten()
        {
            let path = entry.path();
            if path.is_dir()
            {
                results.extend(fetch_files_under(&path, src_ext));
            }
            if let Some(name) = entry.file_name().to_str()
            {
                if name.ends_with(src_ext)
                {
                    results.push(path);
                }
            }
        }
    }
    results
}

#[inline(always)]
pub fn  expand_classpath(paths: &[String]) -> String
{
    let mut entries = Vec::new();
    for path in paths {
        if path.ends_with("/*")
        {
            let dir = &path[..path.len() - 2];
            if let Ok(read_dir) = fs::read_dir(dir)
            {
                for entry in read_dir.flatten()
                {
                    if let Some(ext) = entry.path().extension()
                    {
                        if ext == "jar"
                        {
                            entries.push(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        else
        {
            entries.push(path.clone());
        }
    }
    entries.join(":")
}
