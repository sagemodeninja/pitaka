use std::io::Result;
use std::path::PathBuf;
use std::{fs, io};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub recent_files: Vec<PathBuf>,
    pub last_opened_file: Option<PathBuf>,
}

impl Settings {
    fn new() -> Self {
        Self {
            recent_files: Vec::new(),
            last_opened_file: None,
        }
    }

    pub fn load_or_create() -> Result<Self> {
        let path = data_file_path("settings.json")?;
        if path.exists() { load() } else { create() }
    }

    pub fn update(&self) -> Result<()> {
        save(self)
    }
}

fn data_file_path(filename: &str) -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "garyantier", "pitaka").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine app data directory",
        )
    })?;
    let dir = dirs.data_dir();

    fs::create_dir_all(dir)?;
    Ok(dir.join(filename))
}

fn create() -> Result<Settings> {
    let settings = Settings::new();
    save(&settings)?;
    Ok(settings)
}

fn load() -> Result<Settings> {
    let path = data_file_path("settings.json")?;
    let json = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

fn save(settings: &Settings) -> Result<()> {
    let path = data_file_path("settings.json")?;
    let json = serde_json::to_string(&settings)?;
    fs::write(&path, json)
}
