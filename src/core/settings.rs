use std::fs;
use std::io::Result;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::utils::dir::data_file_dir;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub recent_files: Vec<PathBuf>,
}

impl Settings {
    fn new() -> Self {
        Self {
            recent_files: Vec::new(),
        }
    }

    pub fn load_or_create() -> Result<Self> {
        let path = data_file_dir("settings.json")?;
        if path.exists() { load() } else { create() }
    }

    pub fn add_recent_file(&mut self, path: PathBuf) -> Result<()> {
        if let Some(index) = self.recent_files.iter().position(|item| item == &path) {
            self.recent_files.remove(index);
        }

        self.recent_files.insert(0, path.clone());
        save(self)
    }

    pub fn clear_recents(&mut self) -> Result<()> {
        self.recent_files.clear();
        save(self)
    }
}

fn create() -> Result<Settings> {
    let settings = Settings::new();
    save(&settings)?;
    Ok(settings)
}

fn load() -> Result<Settings> {
    let path = data_file_dir("settings.json")?;
    let json = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

fn save(settings: &Settings) -> Result<()> {
    let path = data_file_dir("settings.json")?;
    let json = serde_json::to_string(&settings)?;
    fs::write(&path, json)
}
