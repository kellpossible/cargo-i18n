use crate::error::{PathError, PathType};
use std::path::Path;

use anyhow::{anyhow, Result};

use walkdir::WalkDir;

pub fn cargo_rerun_if_changed(path: &Path) -> Result<(), PathError> {
    println!(
        "cargo:rerun-if-changed={}",
        path.to_str().ok_or(PathError::not_valid_utf8(
            path,
            "rerun build script if file changed",
            PathType::Directory,
        ))?
    );
    Ok(())
}

/// Ensure that the build script runs every time something within the
/// specified folder changes.
pub fn cargo_rerun_if_dir_changed(path: &Path) -> Result<()> {
    cargo_rerun_if_changed(path)?;

    for result in WalkDir::new(path) {
        match result {
            Ok(entry) => {
                cargo_rerun_if_changed(entry.path())?;
            }
            Err(err) => return Err(anyhow!("error walking directory gui/: {}", err)),
        }
    }

    Ok(())
}
