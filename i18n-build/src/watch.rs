//! Utility functions to use within a `build.rs` build script using
//! this library.

use crate::error::{PathError, PathType};
use std::path::Path;

use anyhow::{anyhow, Result};

use walkdir::WalkDir;

/// Tell `Cargo` to rerun the build script that calls this function
/// (upon rebuild) if the specified file/directory changes.
pub fn cargo_rerun_if_changed(path: &Path) -> Result<(), PathError> {
    println!(
        "cargo:rerun-if-changed={}",
        path.to_str().ok_or_else(|| PathError::not_valid_utf8(
            path,
            "rerun build script if file changed",
            PathType::Directory,
        ))?
    );
    Ok(())
}

/// Tell `Cargo` to rerun the build script that calls this function
/// (upon rebuild) if any of the files/directories within the
/// specified directory changes.
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
