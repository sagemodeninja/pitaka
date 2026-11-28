use std::path::PathBuf;
use std::{fs, io};

use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::utils::dir::data_file_dir;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub recent_files: Vec<PathBuf>,
}

impl Settings {
    pub fn load_or_create() -> io::Result<Self> {
        let path = Self::path()?;

        if path.exists() {
            let json = fs::read_to_string(path)?;
            Ok(from_str(&json)?)
        } else {
            let settings = Self {
                recent_files: Vec::new(),
            };

            settings.save()?;
            Ok(settings)
        }
    }

    pub fn add_recent_file(&mut self, path: PathBuf) -> io::Result<()> {
        if let Some(index) = self.recent_files.iter().position(|item| item == &path) {
            self.recent_files.remove(index);
        }

        self.recent_files.insert(0, path);
        self.save()
    }

    pub fn clear_recents(&mut self) -> io::Result<()> {
        self.recent_files.clear();
        self.save()
    }

    fn save(&self) -> io::Result<()> {
        let path = Self::path()?;
        let json = to_string(&self)?;

        fs::write(&path, json)
    }

    fn path() -> io::Result<PathBuf> {
        data_file_dir("settings.json")
    }
}
