use std::{
    fs,
    io::{ Read },
    path::PathBuf,
};
use serde::{ Serialize, Deserialize };
use toml;
use dirs;
use crate::error::{ Result, with_path };
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub editor: String,
    pub note_db_directory: String,
    pub export_path: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();

        if !config_path.exists() {
            let default_config = Config {
                editor: "vim".to_string(),
                note_db_directory: format!("{}/.local/share/notera", std::env::var("HOME").unwrap_or_else(|_| ".".to_string())),
                export_path: format!("{}/Documents/notera_exports", std::env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            };

            let toml_content = toml::to_string(&default_config)?;

            fs::create_dir_all(config_path.parent().unwrap())
                .map_err(|e| with_path(e, config_path.parent().unwrap().to_path_buf()))?;

            fs::write(&config_path, toml_content)
                .map_err(|e| with_path(e, config_path.clone()))?;

            return Ok(default_config);
        }


        let mut file = fs::File::open(&config_path)
            .map_err(|e| with_path(e, config_path.clone()))?;

        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .map_err(|e| with_path(e, config_path.clone()))?;

        let config = toml::from_str(&contents)?;
        Ok(config)
    }
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("notera");
    path.push("config.toml");
    path
}

