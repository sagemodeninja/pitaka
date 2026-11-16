use std::{fs, io, path::PathBuf};

use directories::{ProjectDirs, UserDirs};

pub fn data_file_dir(filename: &str) -> io::Result<PathBuf> {
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

pub fn documents_dir() -> Option<PathBuf> {
    let user_dirs = UserDirs::new()?;
    user_dirs.document_dir().map(|path| path.to_path_buf())
}
