//! This library contains the configuration stucts (along with their
//! parsing functions) for the
//! [cargo-i18n](https://crates.io/crates/cargo_i18n) tool/system.

use std::fs::read_to_string;
use std::io;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use log::{debug, error};
use serde_derive::Deserialize;
use thiserror::Error;
use toml;

/// An error type for use with the `i18n-config` crate.
#[derive(Debug, Error)]
pub enum I18nConfigError {
    #[error("The specified path is not a crate because there is no Cargo.toml present.")]
    NotACrate(PathBuf),
    #[error("Cannot read file \"{0:?}\" because {1}.")]
    CannotReadFile(PathBuf, #[source] io::Error),
    #[error("Cannot parse Cargo configuration file \"{0:?}\" because {1}.")]
    CannotParseCargoToml(PathBuf, String),
    #[error("Cannot deserialize toml file \"{0:?}\" because {1}.")]
    CannotDeserializeToml(PathBuf, toml::de::Error),
    #[error("Cannot parse i18n configuration file \"{0:?}\" because {1}.")]
    CannotPaseI18nToml(PathBuf, String),
    #[error("There is no i18n configuration file present for the crate {0}.")]
    NoI18nConfig(String),
    #[error("The \"{0}\" is required to be present in the i18n configuration file \"{1}\"")]
    OptionMissingInI18nConfig(String, PathBuf),
    #[error("There is no parent crate for {0}. Required because {1}.")]
    NoParentCrate(String, String),
    #[error(
        "There is no i18n config file present for the parent crate of {0}. Required because {1}."
    )]
    NoParentI18nConfig(String, String),
}

/// Represents a rust crate.
#[derive(Debug, Clone)]
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
    pub fn from<P1: Into<PathBuf>, P2: Into<PathBuf>>(
        path: P1,
        parent: Option<&'a Crate>,
        config_file_path: P2,
    ) -> Result<Crate<'a>, I18nConfigError> {
        let path_into = path.into();

        let config_file_path_into = config_file_path.into();

        let cargo_path = path_into.join("Cargo.toml");

        if !cargo_path.exists() {
            return Err(I18nConfigError::NotACrate(path_into));
        }

        let toml_str = read_to_string(cargo_path.clone())
            .map_err(|err| I18nConfigError::CannotReadFile(cargo_path.clone(), err))?;
        let cargo_toml: toml::Value = toml::from_str(toml_str.as_ref())
            .map_err(|err| I18nConfigError::CannotDeserializeToml(cargo_path.clone(), err))?;

        let package = cargo_toml
            .as_table()
            .ok_or(I18nConfigError::CannotParseCargoToml(cargo_path.clone(), "Cargo.toml needs have sections (such as the \"gettext\" section when using gettext.".to_string()))?
            .get("package")
            .ok_or(I18nConfigError::CannotParseCargoToml(cargo_path.clone(), "Cargo.toml needs to have a \"package\" section.".to_string()))?
            .as_table()
            .ok_or(I18nConfigError::CannotParseCargoToml(cargo_path.clone(),
                "Cargo.toml's \"package\" section needs to contain values.".to_string()
            ))?;

        let name = package
            .get("name")
            .ok_or(I18nConfigError::CannotParseCargoToml(
                cargo_path.clone(),
                "Cargo.toml needs to specify a package name.".to_string(),
            ))?
            .as_str()
            .ok_or(I18nConfigError::CannotParseCargoToml(
                cargo_path.clone(),
                "Cargo.toml's package name needs to be a string.".to_string(),
            ))?;

        let version = package
            .get("version")
            .ok_or(I18nConfigError::CannotParseCargoToml(
                cargo_path.clone(),
                "Cargo.toml needs to specify a package version.".to_string(),
            ))?
            .as_str()
            .ok_or(I18nConfigError::CannotParseCargoToml(
                cargo_path.clone(),
                "Cargo.toml's package version needs to be a string.".to_string(),
            ))?;

        let full_config_file_path = path_into.join(&config_file_path_into);
        let i18n_config = if full_config_file_path.exists() {
            Some(I18nConfig::from_file(&full_config_file_path)?)
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
    pub fn parent_active_config(
        &'a self,
    ) -> Result<Option<(&'a Crate, &'a I18nConfig)>, I18nConfigError> {
        match self.parent {
            Some(parent) => parent.active_config(),
            None => Ok(None),
        }
    }

    /// Identify the config which should be used for this crate, and
    /// the crate (either this crate or one of it's parents)
    /// associated with that config.
    pub fn active_config(&'a self) -> Result<Option<(&'a Crate, &'a I18nConfig)>, I18nConfigError> {
        debug!("Resolving active config for {0}", self);
        match &self.i18n_config {
            Some(config) => {
                match &config.gettext {
                    Some(gettext_config) => {
                        if gettext_config.extract_to_parent {
                            debug!("Resolving active config for {0}, extract_to_parent is true, so attempting to obtain parent config.", self);

                            if !self.parent.is_some() {
                                return Err(I18nConfigError::NoParentCrate(
                                    self.to_string(),
                                    "the gettext extract_to_parent option is active".to_string(),
                                ));
                            }

                            return Ok(Some(self.parent_active_config()?.ok_or_else(|| {
                                I18nConfigError::NoParentI18nConfig(
                                    self.to_string(),
                                    "the gettext extract_to_parent option is active".to_string(),
                                )
                            })?));
                        }
                    }
                    None => {}
                }

                return Ok(Some((self, &config)));
            }
            None => {
                debug!(
                    "{0} has no i18n config, attempting to obtain parent config instead.",
                    self
                );
                return self.parent_active_config();
            }
        };
    }

    /// Get the [I18nConfig](I18nConfig) in this crate, or return an
    /// error if there is none present.
    pub fn config_or_err(&self) -> Result<&I18nConfig, I18nConfigError> {
        match &self.i18n_config {
            Some(config) => Ok(config),
            None => Err(I18nConfigError::NoI18nConfig(self.to_string())),
        }
    }

    /// Get the [GettextConfig](GettextConfig) in this crate, or
    /// return an error if there is none present.
    pub fn gettext_config_or_err(&self) -> Result<&GettextConfig, I18nConfigError> {
        match &self.config_or_err()?.gettext {
            Some(gettext_config) => Ok(gettext_config),
            None => Err(I18nConfigError::OptionMissingInI18nConfig(
                "gettext section".to_string(),
                self.config_file_path.clone(),
            )),
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

    /// Attempt to resolve the parents of this crate which have this
    /// crate listed as a subcrate in their i18n config.
    pub fn find_parent(&self) -> Option<Crate<'a>> {
        let parent_crt = match self
            .path
            .canonicalize()
            .map(|op| op.parent().map(|p| p.to_path_buf()))
            .ok()
            .unwrap_or(None)
        {
            Some(parent_path) => match Crate::from(parent_path, None, "i18n.toml") {
                Ok(parent_crate) => {
                    debug!("Found parent ({0}) of {1}.", parent_crate, self);
                    Some(parent_crate)
                }
                Err(err) => {
                    match err {
                        I18nConfigError::NotACrate(path) => {
                            debug!("The parent of {0} at path {1:?} is not a valid crate with a Cargo.toml", self, path);
                        },
                        _ => {
                            error!(
                                "Error occurred while attempting to resolve parent of {0}: {1}",
                                self, err
                            );
                        },
                    }
                    
                    None
                }
            },
            None => None,
        };

        match parent_crt {
            Some(crt) => match &crt.i18n_config {
                Some(config) => {
                    if config
                        .subcrates
                        .iter()
                        .find(|subcrate_path| {
                            let subcrate_path_canon = match crt.path.join(subcrate_path).canonicalize() {
                                Ok(canon) => canon,
                                Err(err) => {
                                    error!("Error: unable to canonicalize the subcrate path: {0:?} because {1}", subcrate_path, err);
                                    return false;
                                }
                            };

                            let self_path_canon = match self.path.canonicalize() {
                                Ok(canon) => canon,
                                Err(err) => {
                                    error!("Error: unable to canonicalize the crate path: {0:?} because {1}", self.path, err);
                                    return false;
                                }
                            };

                            subcrate_path_canon == self_path_canon
                        })
                        .is_some()
                    {
                        Some(crt)
                    } else {
                        debug!("Parent {0} does not have {1} correctly listed as one of its subcrates (curently: {2:?}) in its i18n config.", crt, self, config.subcrates);
                        None
                    }
                },
                None => {
                    debug!("Parent {0} of {1} does not have an i18n config", crt, self);
                    None
                }
            },
            None => {
                debug!("Could not find a valid parent of {0}.", self);
                None
            }
        }
    }
}

impl<'a> Display for Crate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Crate \"{0}\" at \"{1}\"",
            self.name,
            self.path.to_string_lossy()
        )
    }
}

/// The data structure representing what is stored (and possible to
/// store) within a `i18n.toml` file.
#[derive(Deserialize, Debug, Clone)]
pub struct I18nConfig {
    /// The locale/language identifier of the language used in the source code.
    pub src_locale: String,
    /// The locales that the software will be translated into.
    pub target_locales: Vec<String>,
    /// Specify which subcrates to perform localization within. The
    /// subcrate needs to have its own `i18n.toml`.
    #[serde(default)]
    pub subcrates: Vec<PathBuf>,
    /// The subcomponent of this config relating to gettext, only
    /// present if the gettext localization system will be used.
    pub gettext: Option<GettextConfig>,
}

impl I18nConfig {
    /// Load the config from the specified toml file path.
    pub fn from_file<P: AsRef<Path>>(toml_path: P) -> Result<I18nConfig, I18nConfigError> {
        let toml_path_final: &Path = toml_path.as_ref();
        let toml_str = read_to_string(toml_path_final)
            .map_err(|err| I18nConfigError::CannotReadFile(toml_path_final.to_path_buf(), err))?;
        let config: I18nConfig = toml::from_str(toml_str.as_ref()).map_err(|err| {
            I18nConfigError::CannotDeserializeToml(toml_path_final.to_path_buf(), err)
        })?;
        Ok(config)
    }
}

/// The data structure representing what is stored (and possible to
/// store) within the `gettext` subsection of a `i18n.toml` file.
#[derive(Deserialize, Debug, Clone)]
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
