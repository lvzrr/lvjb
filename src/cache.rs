use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;

pub const CACHE_FILE: &str = "lvjb.lock";

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Cache
{
    pub files:        HashMap<String, String>,
    pub releases:     Vec<Option<(String, String)>>,
    pub url_libs:     Vec<String>,
}

impl Default for Cache
{
    fn default() -> Self
    {
        Cache
        {
            files: HashMap::new(),
            releases: Vec::new(),
            url_libs: Vec::new(),
        }
    }
}

impl Cache
{
    #[inline(always)]
    pub fn load() -> Result<Self, Box<dyn std::error::Error>>
    {
        let content: String = fs::read_to_string(CACHE_FILE)?;
        let cache: Cache = toml::from_str(&content)?;
        Ok(cache)
    }
    #[inline(always)]
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>>
    {
        fs::write(CACHE_FILE, toml::to_string(&self)?)?;
        Ok(())
    }
}
