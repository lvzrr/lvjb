use serde::{Deserialize, Serialize};
use crate::cache::*;
use std::fs;
use toml;

pub const CONF_FILE: &str = "lvjb.toml";

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct PathCnf
{
    pub src:            String,
    pub src_nopkg:      String,
    pub bin:            String,
    pub lib:            String,
    pub test:           String,
    pub docs:           String,
    pub releases:       String,
}

impl Default for PathCnf
{
    fn default() -> Self
    {
        Self
        {
            src:        "src".to_string(),
            src_nopkg:  "default".to_string(),
            bin:        "bin".to_string(),
            lib:        "lib".to_string(),
            test:       "test".to_string(),
            docs:       "docs".to_string(),
            releases:   "releases".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct ArgCnf
{
    pub compilation: Option<Vec<String>>,
    pub runtime:     Option<Vec<String>>,
    pub test:        Option<Vec<String>>,
    pub jvm:         Option<Vec<String>>,
}

impl Default for ArgCnf
{
    fn default() -> Self
    {
        Self
        {
            compilation:    None,
            runtime:        None,
            test:           None,
            jvm:            None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config
{
    pub jar:                String,
    pub compiler:           String,
    pub entry_point:        Option<String>,
    pub src_ext:            String,
    pub classpath:          Vec<String>,
    pub incremental:        bool,
    pub paths:              PathCnf,
    pub args:               ArgCnf,
    pub pre_build_cmds:     Vec<String>,
    pub post_build_cmds:    Vec<String>,
    pub log_level:          u8,
    pub version:            String,
    pub cache:              Cache,

}

impl Default for Config {
    fn default() -> Self
    {
        Config
        {
            jar:                "out".to_string(),
            compiler:           "javac".to_string(),
            entry_point:        None,
            src_ext:            "java".to_string(),
            classpath:          vec!["bin".to_string(), "lib/*".to_string()],
            incremental:        true,
            paths:              PathCnf::default(),
            args:               ArgCnf::default(),
            pre_build_cmds:     Vec::new(),
            post_build_cmds:    Vec::new(),
            log_level:          0,
            version:            "0.0.1".to_string(),
            cache:              match Cache::load()
                                {
                                    Ok(x) => x,
                                    Err(_) =>
                                    {
                                        let t = Cache::default();
                                        let _ = &t.write();
                                        t
                                    },
                                }
        }
    }
}

impl Config
{
    #[inline(always)]
    pub fn load() -> Result<Self, Box<dyn std::error::Error>>
    {
        let content: String = fs::read_to_string(CONF_FILE)?;
        let conf: Config = toml::from_str(&content)?;
        Ok(conf)
    }
    #[inline(always)]
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>>
    {
        fs::write(CONF_FILE, toml::to_string(&self)?)?;
        Ok(())
    }
}
