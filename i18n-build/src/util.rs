//! Utility functions for use with the `i18n_build` library.

use log::debug;
use std::fs::{create_dir_all, remove_file, rename};
use std::path::Path;
use std::process::Command;

use crate::error::PathError;
use anyhow::{ensure, Context, Result};
use tr::tr;

/// Run the specified command, check that it's output was reported as successful.
pub fn run_command_and_check_success(command_name: &str, mut command: Command) -> Result<()> {
    debug!("Running command: {0:?}", &command);
    let output = command
        .spawn()
        .with_context(|| tr!("The \"{0}\" command was unable to start.", command_name))?
        .wait_with_output()
        .with_context(|| {
            tr!(
                "The \"{0}\" command had a problem waiting for output.",
                command_name
            )
        })?;

    ensure!(
        output.status.success(),
        tr!(
            "The \"{0}\" command reported that it was unsuccessful.",
            command_name
        )
    );
    Ok(())
}

/// Check that the given path exists, if it doesn't then throw a
/// [PathError](PathError).
pub fn check_path_exists<P: AsRef<Path>>(path: P) -> Result<(), PathError> {
    if !path.as_ref().exists() {
        Err(PathError::does_not_exist(path.as_ref()))
    } else {
        Ok(())
    }
}

/// Create any of the directories in the specified path if they don't
/// already exist.
pub fn create_dir_all_if_not_exists<P: AsRef<Path>>(path: P) -> Result<(), PathError> {
    if !path.as_ref().exists() {
        create_dir_all(path.as_ref())
            .map_err(|e| PathError::cannot_create_dir(path.as_ref(), e))?;
    }
    Ok(())
}

/// Remove a file if it exists, otherwise return a [PathError#CannotDelete](PathError#CannotDelete).
pub fn remove_file_if_exists<P: AsRef<Path>>(file_path: P) -> Result<(), PathError> {
    if file_path.as_ref().exists() {
        remove_file(file_path.as_ref())
            .map_err(|e| PathError::cannot_delete_file(file_path.as_ref(), e))?;
    }

    Ok(())
}

/// Remove a file, or return a [PathError#CannotDelete](PathError#CannotDelete) if unable to.
pub fn remove_file_or_error<P: AsRef<Path>>(file_path: P) -> Result<(), PathError> {
    remove_file(file_path.as_ref())
        .map_err(|e| PathError::cannot_delete_file(file_path.as_ref(), e))
}

/// Rename a file, or return a [PathError#CannotRename](PathError#CannotRename) if unable to.
pub fn rename_file<P1: AsRef<Path>, P2: AsRef<Path>>(from: P1, to: P2) -> Result<(), PathError> {
    let from_ref = from.as_ref();
    let to_ref = to.as_ref();
    rename(from_ref, to_ref)
        .map_err(|e| PathError::cannot_rename_file(from_ref.to_path_buf(), to_ref.to_path_buf(), e))
}
