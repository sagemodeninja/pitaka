use std::path::PathBuf;

use directories::UserDirs;

pub fn documents_dir() -> Option<PathBuf> {
    let user_dirs = UserDirs::new()?;
    user_dirs.document_dir().map(|path| path.to_path_buf())
}
