use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::env;
use std::error::Error;
use crate::constants::CONFIG_PATH;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub preferred_model: Option<String>,
}

pub fn load_config(config_path: Option<&str>) -> Result<Config, Box<dyn Error>> {
    let config_path = config_path.unwrap_or(CONFIG_PATH);
    let path = Path::new(config_path);

    if !path.exists() {
        if config_path != CONFIG_PATH {
            return Err(format!("Config file not found at: {}", config_path).into());
        }

        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Warning: Could not create config directory: {}", e);
            }
        }

        let config = Config { api_key: None, preferred_model: None };
        let config_str = serde_json::to_string_pretty(&config)?;
        if let Err(e) = fs::write(path, config_str) {
            eprintln!("Warning: Could not write config file: {}", e);
        }
        return Ok(config);
    }

    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}

pub fn get_api_key(config_path: Option<&str>) -> Result<String, Box<dyn Error>> {
    let config = load_config(config_path)?;
    if let Some(api_key) = config.api_key {
        Ok(api_key)
    } else {
        env::var("MISTRAL_API_KEY").map_err(|e| e.into())  // fallback
    }
}

pub fn get_preferred_model(config_path: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
    let config = load_config(config_path)?;
    Ok(config.preferred_model)
}
