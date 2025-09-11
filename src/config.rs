use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_debounce")]
    pub debounce: f32,

    pub sync: Vec<SyncRule>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRule {
    pub name: String,
    pub source: String,
    pub destinations: Vec<String>,

    #[serde(default = "default_sync_on_start")]
    pub sync_on_start: bool,

    #[serde(default = "default_ignore")]
    pub ignore: String,

    #[serde(default = "default_options")]
    pub options: String,
}

fn default_sync_on_start() -> bool {
    true
}

fn default_ignore() -> String {
    ".git\n".into()
}

fn default_options() -> String {
    "-az -e ssh".into()
}

fn default_debounce() -> f32 {
    0.08
}

pub fn read_config(path: &Path) -> Config {
    let toml_str = fs::read_to_string(path).unwrap_or_else(|e| panic!("Error: failed to read config file: {} {e}", path.display()));
    let config: Config = toml::from_str(&toml_str).expect("Failed to parse TOML");
    //println!("Parsed config: {:#?}", config);
    config
}
