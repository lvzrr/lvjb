use crate::config::*;
use std::path::PathBuf;
use std::fs;
use xxhash_rust::xxh3::xxh3_64;

#[inline(always)]
pub fn check_incremental(p: &PathBuf, config: &mut Config) -> bool
{
    let s = p.to_string_lossy().to_string();
    let files = &mut config.cache.files;
    if let Ok(content) = fs::read_to_string(p)
    {
        let hash = xxh3_64(content.as_bytes());
        match files.get(&s)
        {
            Some(prev) if prev.parse::<u64>().unwrap_or_default() == hash => false,
            _ =>
            {
                files.insert(s, hash.to_string());
                true
            }
        }
    }
    else
    {
        true
    }
}
