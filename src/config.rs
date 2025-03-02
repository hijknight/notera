use std::{
    fs,
    io::{ Read },
    path::PathBuf,
};
use serde::{ Serialize, Deserialize };
use toml;
use dirs;
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub editor: String,
    pub note_tmp_directory: String,
}

impl Config {
    pub fn load() -> Self {
        let config_path = get_config_path();

        if !config_path.exists() {
            let default_config = Config {
                editor: "vim".to_string(),
                note_tmp_directory: format!("{}/.local/share/notera", std::env::var("HOME").unwrap_or_else(|_| ".".to_string())),

            };

            let toml_content = toml::to_string(&default_config).expect("Failed to serialize config");

            fs::create_dir_all(config_path.parent().unwrap()).expect("Failed to create config directory");
            fs::write(config_path, toml_content).expect("Failed to write config");

            return default_config;
        }


        let mut file = fs::File::open(config_path).expect("Failed to open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read config file");

        toml::from_str(&contents).expect("Failed to parse config")
    }
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("notera");
    path.push("config.toml");
    path
}