pub mod file;
pub mod settings;

use std::{io::Result, path::PathBuf};

use crate::core::{file::PitakaFile, settings::Settings};

pub struct PitakaCore {
    pub settings: Settings,
    pub active_file: Option<PitakaFile>,
}

impl PitakaCore {
    pub fn init() -> Result<Self> {
        let settings = Settings::load_or_create()?;

        let active_file: Option<PitakaFile> = match &settings.last_opened_file {
            Some(path) => Some(PitakaFile::open(path)?),
            None => None,
        };

        Ok(Self {
            settings,
            active_file,
        })
    }

    pub fn new_file(&mut self, path: PathBuf) -> Result<()> {
        let mut file = PitakaFile::new(&path);
        file.save()?;

        self.active_file = Some(file);
        self.settings.recent_files.push(path.clone());
        self.settings.last_opened_file = Some(path);
        self.settings.update()?;

        Ok(())
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        let file = PitakaFile::open(&path)?;

        self.active_file = Some(file);
        self.settings.recent_files.push(path.clone());
        self.settings.last_opened_file = Some(path);
        self.settings.update()?;

        Ok(())
    }

    pub fn save_file(&mut self) -> Result<()> {
        if let Some(file) = &mut self.active_file {
            file.save()
        } else {
            Ok(())
        }
    }
}
