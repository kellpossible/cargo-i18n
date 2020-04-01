use crate::config::{Crate, I18nConfig};
use crate::error::{PathError, PathType};
use crate::util;

use serde_derive::Deserialize;
use std::ffi::OsStr;
use std::fs::{create_dir_all, remove_file, File};
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use subprocess::Exec;
use walkdir::WalkDir;

#[derive(Deserialize, Debug)]
pub struct GettextConfig {
    /// Path to the output directory, relative to `i18n.toml` of the
    /// crate being localized.
    pub output_dir: Box<Path>,
    /// Whether or not to perform string extraction using the `xtr` tool.
    pub xtr: Option<bool>,
    /// Path to where the pot files will be written to by
    /// [run_xtr()](run_xtr()), and were they will be read from by
    /// [run_msginit()](run_msginit()) and
    /// [run_msgmerge()](run_msgmerge()).
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/pot**.
    pub pot_dir: Option<Box<Path>>,
    /// Path to where the po files will be stored/edited with the
    /// [run_msgmerge()](run_msgmerge()) and
    /// [run_msginit()](run_msginit()) commands, and where they will
    /// be read from with the [run_msgfmt()](run_msgfmt()) command.
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/po**.
    pub po_dir: Option<Box<Path>>,
    /// Path to where the mo files will be written to by the
    /// [run_msgfmt()](run_msgfmt()) command.
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/mo**.
    pub mo_dir: Option<Box<Path>>,
}

pub fn run_xtr(crate_name: &str, src_dir: &Path, pot_dir: &Path) -> Result<()> {
    let mut rs_files: Vec<Box<Path>> = Vec::new();

    for result in WalkDir::new(src_dir) {
        match result {
            Ok(entry) => {
                let path = entry.path().clone();

                match path.extension() {
                    Some(extension) => {
                        if extension.to_str() == Some("rs") {
                            rs_files.push(Box::from(path))
                        }
                    }
                    None => {}
                }
            }
            Err(err) => {
                return Err(anyhow!(
                    "error walking directory {}/src: {}",
                    crate_name,
                    err
                ))
            }
        }
    }

    let mut pot_paths = Vec::new();
    let pot_src_dir = pot_dir.join("src");

    // create pot and pot/tmp if they don't exist
    util::create_dir_all_if_not_exists(&pot_src_dir)?;

    for rs_file_path in rs_files {
        let parent_dir = rs_file_path.parent().context(format!(
            "the rs file {0} is not inside a directory",
            rs_file_path.to_string_lossy()
        ))?;
        let src_dir_relative = parent_dir.strip_prefix(src_dir).map_err(|_| {
            PathError::not_inside_dir(parent_dir, format!("crate {0}/src", crate_name), src_dir)
        })?;
        let file_stem = rs_file_path.file_stem().context(format!(
            "expected rs file path {0} would have a filename",
            rs_file_path.to_string_lossy()
        ))?;

        let pot_file_path = pot_src_dir
            .join(src_dir_relative)
            .join(file_stem)
            .with_extension("pot");

        let pot_dir = pot_file_path.parent().with_context(|| {
            format!(
                "the pot file {0} is not inside a directory",
                pot_file_path.to_string_lossy()
            )
        })?;
        create_dir_all(pot_dir)?;

        // ======= Run the `xtr` command to extract translatable strings =======
        let xtr_command_name = "xtr";
        let mut xtr = Command::new(xtr_command_name);
        xtr.args(&[
            "--package-name",
            "Coster",
            "--package-version",
            "0.1", //TODO: replace this with version from TOML
            "--copyright-holder",
            "Luke Frisken",
            "--msgid-bugs-address",
            "l.frisken@gmail.com",
            "--default-domain",
            crate_name,
            "-o",
            pot_file_path.to_str().ok_or(PathError::not_valid_utf8(
                pot_file_path.clone(),
                "pot",
                PathType::File,
            ))?,
            rs_file_path.to_str().ok_or(PathError::not_valid_utf8(
                rs_file_path.clone(),
                "rs",
                PathType::File,
            ))?,
        ]);

        util::run_command_and_check_success(xtr_command_name, xtr)?;

        pot_paths.push(pot_file_path.to_owned());
    }

    let mut msgcat_args: Vec<Box<OsStr>> = Vec::new();

    for path in pot_paths {
        msgcat_args.push(Box::from(path.as_os_str()));
    }

    let combined_pot_file_path = pot_dir.join("gui.pot");

    if combined_pot_file_path.exists() {
        remove_file(combined_pot_file_path.clone()).context("unable to delete .pot")?;
    }

    let combined_pot_file =
        File::create(combined_pot_file_path).expect("unable to create .pot file");

    // ====== run the `msgcat` command to combine pot files into gui.pot =======
    let msgcat_command_name = "msgcat";
    let msgcat = Exec::cmd(msgcat_command_name)
        .args(msgcat_args.as_slice())
        .stdout(combined_pot_file);

    msgcat.join().context(format!(
        "there was a problem executing the {0} command",
        msgcat_command_name
    ))?;

    Ok(())
}

pub fn run_msginit(
    crate_name: &str,
    i18n_config: &I18nConfig,
    pot_dir: &Path,
    po_dir: &Path,
) -> Result<()> {
    let pot_file_path = pot_dir.join(crate_name).with_extension("pot");

    util::check_path_exists(&pot_file_path)?;

    util::create_dir_all_if_not_exists(po_dir)?;

    let msginit_command_name = "msginit";

    for locale in &i18n_config.target_locales {
        let po_locale_dir = po_dir.join(locale.clone());
        let po_path = po_locale_dir.join(crate_name).with_extension("po");

        if !po_path.exists() {
            create_dir_all(po_locale_dir.clone())
                .map_err(|e| PathError::cannot_create_dir(po_locale_dir, e))?;

            let mut msginit = Command::new(msginit_command_name);
            msginit.args(&[
                format!(
                    "--input={}",
                    pot_file_path.to_str().ok_or(PathError::not_valid_utf8(
                        pot_file_path.clone(),
                        "pot",
                        PathType::File,
                    ))?
                ),
                format!("--locale={}.UTF-8", locale),
                format!(
                    "--output={}",
                    po_path.to_str().ok_or(PathError::not_valid_utf8(
                        po_path.clone(),
                        "po",
                        PathType::File,
                    ))?
                ),
            ]);

            util::run_command_and_check_success(msginit_command_name, msginit)?;
        }
    }

    Ok(())
}

pub fn run_msgmerge(
    crate_name: &str,
    i18n_config: &I18nConfig,
    pot_dir: &Path,
    po_dir: &Path,
) -> Result<()> {
    let pot_file_path = pot_dir.join(crate_name).with_extension("pot");

    util::check_path_exists(&pot_file_path)?;

    let msgmerge_command_name = "msgmerge";

    for locale in &i18n_config.target_locales {
        let po_file_path = po_dir.join(locale).join(crate_name).with_extension("po");

        util::check_path_exists(&po_file_path)?;

        let mut msgmerge = Command::new(msgmerge_command_name);
        msgmerge.args(&[
            "--backup=none",
            "--update",
            po_file_path.to_str().ok_or(PathError::not_valid_utf8(
                po_file_path.clone(),
                "pot",
                PathType::File,
            ))?,
            pot_file_path.to_str().ok_or(PathError::not_valid_utf8(
                pot_file_path.clone(),
                "pot",
                PathType::File,
            ))?,
        ]);

        util::run_command_and_check_success(msgmerge_command_name, msgmerge)?;
    }

    Ok(())
}

pub fn run_msgfmt(
    crate_name: &str,
    i18n_config: &I18nConfig,
    po_dir: &Path,
    mo_dir: &Path,
) -> Result<()> {
    let msgfmt_command_name = "msgfmt";

    for locale in &i18n_config.target_locales {
        let po_file_path = po_dir
            .join(locale.clone())
            .join(crate_name)
            .with_extension("po");

        util::check_path_exists(&po_file_path)?;

        let mo_locale_dir = mo_dir.join(locale);

        if !mo_locale_dir.exists() {
            create_dir_all(mo_locale_dir.clone()).context("trouble creating mo directory")?;
        }

        let mo_file_path = mo_locale_dir.join(crate_name).with_extension("mo");

        let mut msgfmt = Command::new(msgfmt_command_name);
        msgfmt.args(&[
            format!(
                "--output-file={}",
                mo_file_path
                    .to_str()
                    .expect("mo file path is not valid utf-8")
            )
            .as_str(),
            po_file_path
                .to_str()
                .expect("po file path is not valid utf-8"),
        ]);

        util::run_command_and_check_success(msgfmt_command_name, msgfmt)?;
    }

    Ok(())
}

pub fn run(i18n_config: &I18nConfig) -> Result<()> {
    let do_xtr = match i18n_config.gettext_config()?.xtr {
        Some(xtr_value) => xtr_value,
        None => true,
    };

    let crates = vec![Crate::from(Box::from(Path::new(".")))?];

    for subcrate in &crates {
        let crate_dir = subcrate.path.clone();
        let i18n_dir = crate_dir.join("i18n");
        let src_dir = crate_dir.join("src");
        let pot_dir = i18n_dir.join("pot");
        let po_dir = i18n_dir.join("po");
        let mo_dir = i18n_dir.join("mo");

        if do_xtr {
            run_xtr(subcrate.name.as_str(), src_dir.as_path(), pot_dir.as_path())?;
            run_msginit(
                subcrate.name.as_str(),
                i18n_config,
                pot_dir.as_path(),
                po_dir.as_path(),
            )?;
            run_msgmerge(
                subcrate.name.as_str(),
                i18n_config,
                pot_dir.as_path(),
                po_dir.as_path(),
            )?;
        }

        run_msgfmt(
            subcrate.name.as_str(),
            i18n_config,
            po_dir.as_path(),
            mo_dir.as_path(),
        )?;
    }

    return Ok(());
}
