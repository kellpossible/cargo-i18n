//! This library contains the configuration stucts (along with their
//! parsing functions) for the
//! [cargo-i18n](https://crates.io/crates/cargo_i18n) tool/system.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_derive::Deserialize;
use toml;

/// Represents a rust crate.
#[derive(Debug)]
pub struct Crate<'a> {
    /// The name of the crate.
    pub name: String,
    /// The version of the crate.
    pub version: String,
    /// The path to the crate.
    pub path: PathBuf,
    /// Path to the parent crate which is triggering the localization
    /// for this crate.
    pub parent: Option<&'a Crate<'a>>,
    /// The file path expected to be used for `i18n_config` relative to this crate's root.
    pub config_file_path: PathBuf,
    /// The localization config for this crate (if it exists).
    pub i18n_config: Option<I18nConfig>,
}

impl<'a> Crate<'a> {
    /// Read crate from `Cargo.toml` i18n config using the
    /// `config_file_path` (if there is one).
    pub fn from<P: Into<PathBuf>>(
        path: P,
        parent: Option<&'a Crate>,
        config_file_path: P,
    ) -> Result<Crate<'a>> {
        let path_into = path.into();
        let config_file_path_into = config_file_path.into();

        let cargo_path = path_into.join("Cargo.toml");
        let toml_str = read_to_string(cargo_path.clone())
            .with_context(|| format!("trouble reading {0:?}", cargo_path))?;
        let cargo_toml: toml::Value = toml::from_str(toml_str.as_ref())
            .with_context(|| format!("trouble parsing {0:?}", cargo_path))?;

        let package = cargo_toml
            .as_table()
            .ok_or(anyhow!("Cargo.toml needs have sections (such as the \"gettext\" section when using gettext."))?
            .get("package")
            .ok_or(anyhow!("Cargo.toml needs to have a \"package\" section."))?
            .as_table()
            .ok_or(anyhow!(
                "Cargo.toml's \"package\" section needs to contain values."
            ))?;

        let name = package
            .get("name")
            .ok_or(anyhow!("Cargo.toml needs to specify a package name."))?
            .as_str()
            .ok_or(anyhow!("Cargo.toml's package name needs to be a string."))?;

        let version = package
            .get("version")
            .ok_or(anyhow!("Cargo.toml needs to specify a package version."))?
            .as_str()
            .ok_or(anyhow!(
                "Cargo.toml's package version needs to be a string."
            ))?;

        let full_config_file_path = path_into.join(&config_file_path_into);
        let i18n_config = if full_config_file_path.exists() {
            Some(
                I18nConfig::from_file(&full_config_file_path).with_context(|| {
                    format!(
                        "Cannot load i18n config file: {0}.",
                        full_config_file_path.to_string_lossy()
                    )
                })?,
            )
        } else {
            None
        };

        Ok(Crate {
            name: String::from(name),
            version: String::from(version),
            path: path_into,
            parent,
            config_file_path: config_file_path_into,
            i18n_config,
        })
    }

    /// The name of the module/library used for this crate. Replaces
    /// `-` characters with `_` in the crate name.
    pub fn module_name(&self) -> String {
        self.name.replace("-", "_")
    }

    /// If there is a parent, get it's
    /// [I18nConfig#active_config()](I18nConfig#active_config()),
    /// otherwise return None.
    pub fn parent_active_config(&'a self) -> Option<(&'a Crate, &'a I18nConfig)> {
        match self.parent {
            Some(parent) => parent.active_config(),
            None => None,
        }
    }

    /// Identify the config which should be used for this crate, and
    /// the crate (either this crate or one of it's parents)
    /// associated with that config.
    pub fn active_config(&'a self) -> Option<(&'a Crate, &'a I18nConfig)> {
        match &self.i18n_config {
            Some(config) => {
                match &config.gettext {
                    Some(gettext_config) => {
                        if gettext_config.extract_to_parent {
                            return self.parent_active_config();
                        }
                    }
                    None => {}
                }

                return Some((self, &config));
            }
            None => {
                return self.parent_active_config();
            }
        };
    }

    /// Get the [I18nConfig](I18nConfig) in this crate, or return an
    /// error if there is none present.
    pub fn config_or_err(&self) -> Result<&I18nConfig> {
        match &self.i18n_config {
            Some(config) => Ok(config),
            None => Err(anyhow!(format!(
                "There is no i18n config file \"{0}\" present in this crate \"{1}\".",
                self.config_file_path.to_string_lossy(),
                self.name
            ))),
        }
    }

    /// Get the [GettextConfig](GettextConfig) in this crate, or
    /// return an error if there is none present.
    pub fn gettext_config_or_err(&self) -> Result<&GettextConfig> {
        match &self.config_or_err()?.gettext {
            Some(gettext_config) => Ok(gettext_config),
            None => Err(anyhow!(format!(
                "There is no gettext config available the i18n config \"{0}\".",
                self.config_file_path.to_string_lossy(),
            ))),
        }
    }

    /// If this crate has a parent, check whether the parent wants to
    /// collate subcrates string extraction, as per the parent's
    /// [GettextConfig#collate_extracted_subcrates](GettextConfig#collate_extracted_subcrates).
    /// This also requires that the current crate's [GettextConfig#extract_to_parent](GettextConfig#extract_to_parent)
    /// is **true**.
    ///
    /// Returns **false** if there is no parent or the parent has no gettext config.
    pub fn collated_subcrate(&self) -> bool {
        let parent_extract_to_subcrate = self
            .parent
            .map(|parent_crate| {
                parent_crate
                    .gettext_config_or_err()
                    .map(|parent_gettext_config| parent_gettext_config.collate_extracted_subcrates)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        let extract_to_parent = self
            .gettext_config_or_err()
            .map(|gettext_config| gettext_config.extract_to_parent)
            .unwrap_or(false);

        return parent_extract_to_subcrate && extract_to_parent;
    }
}

/// The data structure representing what is stored (and possible to
/// store) within a `i18n.toml` file.
#[derive(Deserialize, Debug)]
pub struct I18nConfig {
    /// The locale/language identifier of the language used in the source code.
    pub src_locale: String,
    /// The locales that the software will be translated into.
    pub target_locales: Vec<String>,
    /// Specify which subcrates to perform localization within. The
    /// subcrate needs to have its own `i18n.toml`.
    pub subcrates: Option<Vec<PathBuf>>,
    /// The subcomponent of this config relating to gettext, only
    /// present if the gettext localization system will be used.
    pub gettext: Option<GettextConfig>,
}

impl I18nConfig {
    /// Load the config from the specified toml file path.
    pub fn from_file<P: AsRef<Path>>(toml_path: P) -> Result<I18nConfig> {
        let toml_path_final: &Path = toml_path.as_ref();
        let toml_str = read_to_string(toml_path_final).with_context(|| {
            format!(
                "Trouble reading file \"{0}\".",
                toml_path_final.to_string_lossy()
            )
        })?;
        let config: I18nConfig = toml::from_str(toml_str.as_ref()).with_context(|| {
            format!(
                "There was an error while parsing an i18n config file \"{0}\".",
                toml_path_final.to_string_lossy()
            )
        })?;
        Ok(config)
    }
}

/// The data structure representing what is stored (and possible to
/// store) within the `gettext` subsection of a `i18n.toml` file.
#[derive(Deserialize, Debug)]
pub struct GettextConfig {
    /// Path to the output directory, relative to `i18n.toml` of the
    /// crate being localized.
    pub output_dir: PathBuf,
    // If this crate is being localized as a subcrate, store the
    // localization artifacts with the parent crate's output.
    // Currently crates which contain subcrates with duplicate names
    // are not supported.
    //
    // By default this is **false**.
    #[serde(default)]
    pub extract_to_parent: bool,
    // If a subcrate has extract_to_parent set to true,
    // then merge the output pot file of that subcrate into this
    // crate's pot file.
    //
    // By default this is **false**.
    #[serde(default)]
    pub collate_extracted_subcrates: bool,
    /// Set the copyright holder for the generated files.
    pub copyright_holder: Option<String>,
    /// The reporting address for msgid bugs. This is the email
    /// address or URL to which the translators shall report bugs in
    /// the untranslated strings.
    pub msgid_bugs_address: Option<String>,
    /// Whether or not to perform string extraction using the `xtr` command.
    pub xtr: Option<bool>,
    /// Path to where the pot files will be written to by the `xtr`
    /// command, and were they will be read from by `msginit` and
    /// `msgmerge`.
    pot_dir: Option<PathBuf>,
    /// Path to where the po files will be stored/edited with the
    /// `msgmerge` and `msginit` commands, and where they will be read
    /// from with the `msgfmt` command.
    po_dir: Option<PathBuf>,
    /// Path to where the mo files will be written to by the
    /// `msgfmt` command.
    mo_dir: Option<PathBuf>,
}

impl GettextConfig {
    /// Path to where the pot files will be written to by the `xtr`
    /// command, and were they will be read from by `msginit` and
    /// `msgmerge`.
    ///
    /// By default this is
    /// **[output_dir](GettextConfig::output_dir)/pot**.
    pub fn pot_dir(&self) -> PathBuf {
        // match self.pot_dir {
        //     Some(pot_dir) => pot_dir,
        //     None => {
        //         panic!("panic");
        //     },
        // }
        self.pot_dir.clone().unwrap_or(self.output_dir.join("pot"))
    }

    /// Path to where the po files will be stored/edited with the
    /// `msgmerge` and `msginit` commands, and where they will
    /// be read from with the `msgfmt` command.
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/po**.
    pub fn po_dir(&self) -> PathBuf {
        self.po_dir.clone().unwrap_or(self.output_dir.join("po"))
    }

    /// Path to where the mo files will be written to by the `msgfmt` command.
    ///
    /// By default this is
    /// **[output_dir](GettextConfig::output_dir)/mo**.
    pub fn mo_dir(&self) -> PathBuf {
        self.mo_dir.clone().unwrap_or(self.output_dir.join("mo"))
    }
}
