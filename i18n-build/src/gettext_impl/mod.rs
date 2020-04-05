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
use tr::tr;
use walkdir::WalkDir;

#[derive(Deserialize, Debug)]
pub struct GettextConfig {
    /// Path to the output directory, relative to `i18n.toml` of the
    /// crate being localized.
    pub output_dir: Box<Path>,
    // If this crate is being localized as a subcrate, store the
    // localization artifacts with the parent crate's output.
    // Currently crates which contain subcrates with duplicate names
    // are not supported.
    //
    // By default this will be treated as **false**.
    pub extract_to_parent: Option<bool>,
    /// Set the copyright holder for the generated files.
    pub copyright_holder: Option<String>,
    /// The reporting address for msgid bugs. This is the email
    /// address or URL to which the translators shall report bugs in
    /// the untranslated strings.
    pub msgid_bugs_address: Option<String>,
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

    let combined_pot_file_path = pot_dir.join(crt.module_name()).with_extension("pot");

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

    msgcat.join().with_context(|| {
        tr!(
            "There was a problem executing the \"{0}\" command",
            msgcat_command_name
        )
    })?;

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
    let (config_crate, i18n_config) = crt
        .active_config()
        .expect("expected that there would be an active config");

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
        Some(config) => match &config.subcrates {
            Some(subcrate_paths) => {
                let subcrates: Result<Vec<Crate>, anyhow::Error> = subcrate_paths
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
                    let subcrate_path_strings: Vec<String> = subcrate_paths
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect();
                    
                    tr!(
                        "There was a problem parsing one of the subcrates: {0}.",
                        subcrate_path_strings.join(", ")
                    )
                })?
            }
            None => vec![],
        },
        None => vec![],
    };

    let i18n_dir = config_crate.path.join("i18n");
    let src_dir = crt.path.join("src");
    let pot_dir = i18n_dir.join("pot");
    let po_dir = i18n_dir.join("po");
    let mo_dir = i18n_dir.join("mo");

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
        run_msginit(crt, i18n_config, pot_dir.as_path(), po_dir.as_path())?;
        run_msgmerge(crt, i18n_config, pot_dir.as_path(), po_dir.as_path())?;
    }

    run_msgfmt(crt, i18n_config, po_dir.as_path(), mo_dir.as_path())?;

    for subcrate in &subcrates {
        run(subcrate)?;
    }

    return Ok(());
}
