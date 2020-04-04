use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

use crate::error::PathError;
use anyhow::{anyhow, ensure, Context, Result};
use tr::tr;

pub fn run_command_and_check_success(command_name: &str, mut command: Command) -> Result<()> {
    let output = command
        .spawn()
        .with_context(|| tr!("the {0} command was unable to start", command_name))?
        .wait_with_output()
        .with_context(|| {
            tr!(
                "the {0} command had a problem waiting for output",
                command_name
            )
        })?;

    ensure!(
        output.status.success(),
        tr!(
            "the {0} command reported that it was unsuccessful",
            command_name
        )
    );
    Ok(())
}

pub fn check_path_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        Err(anyhow!(PathError::does_not_exist(path)))
    } else {
        Ok(())
    }
}

pub fn create_dir_all_if_not_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_dir_all(path.clone()).map_err(|e| PathError::cannot_create_dir(path.clone(), e))?;
    }
    Ok(())
}
