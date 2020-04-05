use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

use crate::error::PathError;
use anyhow::{ensure, Context, Result};
use tr::tr;

/// Run the specified command, check that it's output was reported as successful.
pub fn run_command_and_check_success(command_name: &str, mut command: Command) -> Result<()> {
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
pub fn check_path_exists(path: &Path) -> Result<(), PathError> {
    if !path.exists() {
        Err(PathError::does_not_exist(path))
    } else {
        Ok(())
    }
}

/// Create any of the directories in the specified path if they don't
/// already exist.
pub fn create_dir_all_if_not_exists(path: &Path) -> Result<(), PathError> {
    if !path.exists() {
        create_dir_all(path.clone()).map_err(|e| PathError::cannot_create_dir(path.clone(), e))?;
    }
    Ok(())
}
