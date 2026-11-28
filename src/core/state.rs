use std::io;
use std::path::PathBuf;

use crate::core::{PitakaVault, Settings};

pub struct PitakaState {
    pub settings: Settings,
    pub active_vault: Option<PitakaVault>,
}

impl PitakaState {
    pub fn init() -> io::Result<Self> {
        let settings = Settings::load_or_create()?;

        let active_vault: Option<PitakaVault> = match &settings.recent_files.first() {
            Some(path) => Some(PitakaVault::open(path)?),
            None => None,
        };

        Ok(Self {
            settings,
            active_vault,
        })
    }

    pub fn new_vault(&mut self, name: impl Into<String>, path: PathBuf) -> io::Result<()> {
        let vault = PitakaVault::create(name, &path)?;
        self.active_vault(vault, path)
    }

    pub fn open_doc(&mut self, path: PathBuf) -> io::Result<()> {
        let vault = PitakaVault::open(&path)?;
        self.active_vault(vault, path)
    }

    pub fn save_doc(&mut self) -> io::Result<()> {
        if let Some(vault) = &mut self.active_vault {
            vault.save()
        } else {
            Ok(())
        }
    }

    fn active_vault(&mut self, vault: PitakaVault, path: PathBuf) -> io::Result<()> {
        self.active_vault = Some(vault);
        self.settings.add_recent_file(path)
    }
}
