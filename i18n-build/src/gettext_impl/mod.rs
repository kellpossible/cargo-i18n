//! This module contains the implementation for localizing using the
//! `gettext` localization system.

use crate::error::{PathError, PathType};
use crate::util;
use i18n_config::{Crate, GettextConfig, I18nConfig, I18nConfigError};

use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use subprocess::Exec;
use tr::tr;
use walkdir::WalkDir;

/// Run the `xtr` command (<https://crates.io/crates/xtr/>) in order
/// to extract the translateable strings from the crate.
///
/// `src_dir` is the directory where the Rust source code is located
/// relative to the crate path.
///
/// `pot_dir` is the directory where the output `pot` files will be
/// stored.
///
/// `prepend_crate_path` is whether or not to prepend the path of the
/// crate to directory where the intermediate `pot` files will be
/// stored within the `pot_dir`.
pub fn run_xtr(
    crt: &Crate,
    gettext_config: &GettextConfig,
    src_dir: &Path,
    pot_dir: &Path,
    prepend_crate_path: bool,
) -> Result<()> {
    info!(
        "Performing string extraction with `xtr` for crate \"{0}\"",
        crt.path.to_string_lossy()
    );
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
            Err(err) => return Err(anyhow!("error walking directory {}/src: {}", crt.name, err)),
        }
    }

    let mut pot_paths = Vec::new();

    let pot_src_dir = if prepend_crate_path {
        pot_dir.join(&crt.path).join("src")
    } else {
        pot_dir.join("src")
    };

    // create pot and pot/tmp if they don't exist
    util::create_dir_all_if_not_exists(&pot_src_dir)?;

    for rs_file_path in rs_files {
        let parent_dir = rs_file_path.parent().context(format!(
            "the rs file {0} is not inside a directory",
            rs_file_path.to_string_lossy()
        ))?;
        let src_dir_relative = parent_dir.strip_prefix(src_dir).map_err(|_| {
            PathError::not_inside_dir(parent_dir, format!("crate {0}/src", crt.name), src_dir)
        })?;
        let file_stem = rs_file_path.file_stem().context(format!(
            "expected rs file path {0} would have a filename",
            rs_file_path.to_string_lossy()
        ))?;

        let pot_file_path = pot_src_dir
            .join(src_dir_relative)
            .join(file_stem)
            .with_extension("pot");

        util::create_dir_all_if_not_exists(pot_file_path.parent().with_context(|| {
            format!(
                "Expected that pot file path \"{0}\" would be inside a directory (have a parent)",
                &pot_file_path.to_string_lossy()
            )
        })?)?;

        // ======= Run the `xtr` command to extract translatable strings =======
        let xtr_command_name = "xtr";
        let mut xtr = Command::new(xtr_command_name);

        match &gettext_config.copyright_holder {
            Some(copyright_holder) => {
                xtr.args(&["--copyright-holder", copyright_holder.as_str()]);
            }
            None => {}
        }

        match &gettext_config.msgid_bugs_address {
            Some(msgid_bugs_address) => {
                xtr.args(&["--msgid-bugs-address", msgid_bugs_address.as_str()]);
            }
            None => {}
        }

        xtr.args(&[
            "--package-name",
            crt.name.as_str(),
            "--package-version",
            crt.version.as_str(),
            "--default-domain",
            crt.module_name().as_str(),
            "--add-location",
            gettext_config.add_location.to_str(),
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

    for path in &pot_paths {
        msgcat_args.push(Box::from(path.as_os_str()));
    }

    let combined_pot_file_path = crate_module_pot_file_path(crt, pot_dir)?;

    run_msgcat(&pot_paths, &combined_pot_file_path)
        .context("There was a problem while trying to run the \"msgcat\" command.")?;

    Ok(())
}

fn crate_module_pot_file_path<'a, P: AsRef<Path>>(crt: &Crate<'a>, pot_dir: P) -> Result<PathBuf> {
    Ok(pot_dir
        .as_ref()
        .join(crt.module_name())
        .with_extension("pot"))
}

/// Run the gettext utils `msgcat` command to concatinate pot files
/// into a single pot file.
pub fn run_msgcat<P: AsRef<Path>, I: IntoIterator<Item = P>>(
    input_pot_paths: I,
    output_pot_path: P,
) -> Result<()> {
    let input_pot_paths_iter = input_pot_paths.into_iter();

    let mut input_pot_paths_strings: Vec<String> = Vec::new();
    let mut msgcat_args: Vec<Box<OsStr>> = Vec::new();

    let mut output_in_input = false;
    for input_path in input_pot_paths_iter {
        let input_path_ref = input_path.as_ref();
        input_pot_paths_strings.push(input_path_ref.to_string_lossy().to_string());
        msgcat_args.push(Box::from(input_path_ref.as_os_str()));
        output_in_input |= input_path_ref == output_pot_path.as_ref();
    }

    info!(
        "Concatinating pot files {0:?} with `msgcat` into \"{1}\"",
        input_pot_paths_strings,
        output_pot_path.as_ref().to_string_lossy()
    );

    let interim_output_pot_path = if output_in_input {
        output_pot_path.as_ref().with_extension("pot.tmp")
    } else {
        output_pot_path.as_ref().to_path_buf()
    };

    util::create_dir_all_if_not_exists(
        &interim_output_pot_path
            .parent()
            .expect("expected there to be a parent to the interim output pot path"),
    )?;

    util::remove_file_if_exists(&interim_output_pot_path)?;

    let output_pot_file = File::create(&interim_output_pot_path)
        .map_err(|e| PathError::cannot_create_file(&interim_output_pot_path, e))?;

    let msgcat_command_name = "msgcat";
    let msgcat = Exec::cmd(msgcat_command_name)
        .args(msgcat_args.as_slice())
        .stdout(output_pot_file);

    debug!("Running command: {0:?}", msgcat);

    msgcat.join().with_context(|| {
        tr!(
            "There was a problem executing the \"{0}\" command",
            msgcat_command_name
        )
    })?;

    if output_in_input {
        util::remove_file_if_exists(&output_pot_path)?;
        util::rename_file(&interim_output_pot_path, &output_pot_path)?;
    }

    Ok(())
}

/// Run the gettext `msginit` command to create a new `po` file.
///
/// `pot_dir` is the directory where the input `pot` files are stored.
///
/// `po_dir` is the directory where the output `po` files will be
/// stored.
pub fn run_msginit(
    crt: &Crate,
    i18n_config: &I18nConfig,
    pot_dir: &Path,
    po_dir: &Path,
) -> Result<()> {
    info!(
        "Initializing new po files with `msginit` for crate \"{0}\"",
        crt.path.to_string_lossy()
    );
    let pot_file_path = pot_dir.join(crt.module_name()).with_extension("pot");

    util::check_path_exists(&pot_file_path)?;

    util::create_dir_all_if_not_exists(po_dir)?;

    let msginit_command_name = "msginit";

    for locale in &i18n_config.target_locales {
        let po_locale_dir = po_dir.join(locale.clone());
        let po_path = po_locale_dir.join(crt.module_name()).with_extension("po");

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

/// Run the gettext `msgmerge` command to update the `po` files with
/// new/deleted messages from the source `pot` files.
///
/// `pot_dir` is the directory where the input `pot` files are stored.
///
/// `po_dir` is the directory where the `po` files are stored.
pub fn run_msgmerge(
    crt: &Crate,
    i18n_config: &I18nConfig,
    pot_dir: &Path,
    po_dir: &Path,
) -> Result<()> {
    info!(
        "Merging message changes in pot files to po files with `msgmerge` for crate \"{0}\"",
        crt.path.to_string_lossy()
    );
    let pot_file_path = pot_dir.join(crt.module_name()).with_extension("pot");

    util::check_path_exists(&pot_file_path)?;

    let msgmerge_command_name = "msgmerge";

    for locale in &i18n_config.target_locales {
        let po_file_path = po_dir
            .join(locale)
            .join(crt.module_name())
            .with_extension("po");

        util::check_path_exists(&po_file_path)?;

        let mut msgmerge = Command::new(msgmerge_command_name);
        msgmerge.args(&[
            "--silent",
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

/// Run the gettext `msgfmt` command to compile the `po` files into
/// binary `mo` files.
///
/// `po_dir` is the directory where the input `po` files are stored.
///
/// `mo_dir` is the directory where the output `mo` files will be stored.
pub fn run_msgfmt(
    crt: &Crate,
    i18n_config: &I18nConfig,
    po_dir: &Path,
    mo_dir: &Path,
) -> Result<()> {
    info!(
        "Compiling po files to mo files with `msgfmt` for crate \"{0}\"",
        crt.path.to_string_lossy()
    );
    let msgfmt_command_name = "msgfmt";

    for locale in &i18n_config.target_locales {
        let po_file_path = po_dir
            .join(locale.clone())
            .join(crt.module_name())
            .with_extension("po");

        util::check_path_exists(&po_file_path)?;

        let mo_locale_dir = mo_dir.join(locale);

        if !mo_locale_dir.exists() {
            create_dir_all(mo_locale_dir.clone()).context("trouble creating mo directory")?;
        }

        let mo_file_path = mo_locale_dir.join(crt.module_name()).with_extension("mo");

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

/// Run the gettext i18n build process for the provided crate. The
/// crate must have an i18n config containing a gettext config.
///
/// This function is recursively executed for each subcrate.
pub fn run<'a>(crt: &'a Crate) -> Result<()> {
    info!(
        "Localizing crate \"{0}\" using the gettext system",
        crt.path.to_string_lossy()
    );
    let (config_crate, i18n_config) = crt.active_config()?.expect(&format!(
        "expected that there would be an active config for the crate: \"{0}\" at \"{1}\"",
        crt.name,
        crt.path.to_string_lossy()
    ));

    let gettext_config = config_crate
        .gettext_config_or_err()
        .expect("expected gettext config to be present");

    let do_xtr = match config_crate.gettext_config_or_err()?.xtr {
        Some(xtr_value) => xtr_value,
        None => true,
    };

    // We don't use the i18n_config (which potentially comes from the
    // parent crate )to get the subcrates, because this would result
    // in an infinite loop.
    let subcrates: Vec<Crate> = match &crt.i18n_config {
        Some(config) => {
            let subcrates: Result<Vec<Crate>, I18nConfigError> = config.subcrates
                .iter()
                .map(|subcrate_path| {
                    Crate::from(
                        subcrate_path.clone(),
                        Some(crt),
                        crt.config_file_path.clone(),
                    )
                })
                .collect();

            subcrates.with_context(|| {
                let subcrate_path_strings: Vec<String> = config.subcrates
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect();

                tr!(
                    "There was a problem parsing one of the subcrates: \"{0}\".",
                    subcrate_path_strings.join(", ")
                )
            })?
        },
        None => vec![],
    };

    let src_dir = crt.path.join("src");
    let pot_dir = config_crate.path.join(gettext_config.pot_dir());
    let po_dir = config_crate.path.join(gettext_config.po_dir());
    let mo_dir = config_crate.path.join(gettext_config.mo_dir());

    // perform string extraction if required
    if do_xtr {
        let prepend_crate_path =
            crt.path.canonicalize().unwrap() != config_crate.path.canonicalize().unwrap();
        run_xtr(
            crt,
            &gettext_config,
            src_dir.as_path(),
            pot_dir.as_path(),
            prepend_crate_path,
        )?;
    }

    // figure out where there are any subcrates which need their output
    // pot files concatinated with this crate's pot file
    let mut concatinate_crates = vec![];
    for subcrate in &subcrates {
        run(subcrate)?;
        if subcrate.collated_subcrate() {
            concatinate_crates.push(subcrate);
        }
    }

    // Perform the concatination (if there are any required)
    if concatinate_crates.len() > 0 {
        assert!(crt.gettext_config_or_err()?.collate_extracted_subcrates == true);
        concatinate_crates.insert(0, crt);

        let concatinate_crate_paths_result: Result<Vec<PathBuf>, _> = concatinate_crates
            .iter()
            .map(|concat_crt: &&Crate| crate_module_pot_file_path(concat_crt, &pot_dir))
            .collect();

        let concatinate_crate_paths = concatinate_crate_paths_result?;

        let output_pot_path = crate_module_pot_file_path(crt, &pot_dir)?;
        run_msgcat(concatinate_crate_paths, output_pot_path)?;

        // remove this crate from the list because we don't want to delete it's pot file
        concatinate_crates.remove(0);

        for subcrate in concatinate_crates {
            let subcrate_output_pot_path = crate_module_pot_file_path(subcrate, &pot_dir)?;
            util::remove_file_or_error(subcrate_output_pot_path)?;
        }
    }

    if !(crt.collated_subcrate()) {
        run_msginit(crt, i18n_config, pot_dir.as_path(), po_dir.as_path())?;
        run_msgmerge(crt, i18n_config, pot_dir.as_path(), po_dir.as_path())?;
        run_msgfmt(crt, i18n_config, po_dir.as_path(), mo_dir.as_path())?;
    }

    return Ok(());
}
