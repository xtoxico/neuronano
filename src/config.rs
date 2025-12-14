use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_key: String,
}

impl Config {
    pub fn default() -> Self {
        Self {
            api_key: String::new(),
        }
    }

    pub fn load() -> Result<Self> {
        if let Ok(content) = fs::read_to_string("config.json") {
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create("config.json")?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
