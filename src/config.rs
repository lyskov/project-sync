use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_debounce")]
    pub debounce: f32,

    #[serde(default)] // missing field → false
    pub verbose: bool,

    pub sync: Vec<SyncRuleToml>,

    #[serde(skip)]
    pub config_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct SyncRuleToml {
    pub name: String,
    pub source: String,
    pub destinations: Vec<String>,

    #[serde(default = "default_sync_on_start")]
    pub sync_on_start: bool,

    #[serde(default = "default_ignore")]
    pub ignore: String,

    #[serde(default)]
    pub options: String,
}

fn default_sync_on_start() -> bool {
    false
}

fn default_ignore() -> String {
    ".git\n".into()
}

// fn default_options() -> String {
//     "-az -e ssh".into()
// }

fn default_debounce() -> f32 {
    0.08
}

impl Config {
    pub fn from_config_path(path: &Path) -> Self {
        let toml_str = fs::read_to_string(path).unwrap_or_else(|e| panic!("Error: failed to read config file: {} {e}", path.display()));
        let mut config: Config = toml::from_str(&toml_str).expect("Failed to parse TOML");
        //println!("Parsed config: {:#?}", config);

        config.config_path = path.into();
        config
    }
    pub fn retain_destinations(&mut self, filter: &str) {
        for s in self.sync.iter_mut() {
            s.destinations.retain(|dest| dest.contains(filter));
        }
    }
}
