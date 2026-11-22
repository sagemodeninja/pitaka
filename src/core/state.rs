use std::io;
use std::path::PathBuf;

use crate::core::{settings::Settings, vault::PitakaVault};

pub struct PitakaState {
    pub settings: Settings,
    pub vault: Option<PitakaVault>,
}

impl PitakaState {
    pub fn init() -> io::Result<Self> {
        let settings = Settings::load_or_create()?;

        let vault: Option<PitakaVault> = match &settings.recent_files.first() {
            Some(path) => Some(PitakaVault::open(path)?),
            None => None,
        };

        Ok(Self { settings, vault })
    }

    pub fn new_vault(&mut self, name: impl Into<String>, path: PathBuf) -> io::Result<()> {
        let vault = PitakaVault::create(name, &path)?;

        self.vault = Some(vault);
        self.settings.add_recent_file(path)?;

        Ok(())
    }

    pub fn open_doc(&mut self, path: PathBuf) -> io::Result<()> {
        let vault = PitakaVault::open(&path)?;

        self.vault = Some(vault);
        self.settings.add_recent_file(path)?;

        Ok(())
    }

    pub fn save_doc(&mut self) -> io::Result<()> {
        if let Some(vault) = &mut self.vault {
            vault.save()
        } else {
            Ok(())
        }
    }
}
