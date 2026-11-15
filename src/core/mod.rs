pub mod document;
pub mod settings;

use std::{io::Result, path::PathBuf};

use crate::core::{document::PitakaDocument, settings::Settings};

pub struct PitakaCore {
    pub settings: Settings,
    pub document: Option<PitakaDocument>,
}

impl PitakaCore {
    pub fn init() -> Result<Self> {
        let settings = Settings::load_or_create()?;

        let document: Option<PitakaDocument> = match &settings.recent_files.first() {
            Some(path) => Some(PitakaDocument::open(path)?),
            None => None,
        };

        Ok(Self { settings, document })
    }

    pub fn new_doc(&mut self, path: PathBuf) -> Result<()> {
        let document = PitakaDocument::create(&path)?;

        self.document = Some(document);
        self.settings.add_recent_file(path)?;

        Ok(())
    }

    pub fn open_doc(&mut self, path: PathBuf) -> Result<()> {
        let document = PitakaDocument::open(&path)?;

        self.document = Some(document);
        self.settings.add_recent_file(path)?;

        Ok(())
    }

    pub fn save_doc(&mut self) -> Result<()> {
        if let Some(document) = &mut self.document {
            document.save()
        } else {
            Ok(())
        }
    }
}
