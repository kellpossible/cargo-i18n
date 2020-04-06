use std::fs::read_to_string;
use std::path::{PathBuf, Path};

use anyhow::{anyhow, Context, Result};
use serde_derive::Deserialize;
use toml;

/// Represents a rust crate.
pub struct Crate<'a> {
    /// The name of the crate.
    pub name: String,
    /// The version of the crate.
    pub version: String,
    /// The path to the crate.
    pub path: Box<Path>,
    /// Path to the parent crate which is triggering the localization
    /// for this crate.
    pub parent: Option<&'a Crate<'a>>,
    /// The file path expected to be used for `i18n_config` relative to this crate's root.
    pub config_file_path: Box<Path>,
    /// The localization config for this crate (if it exists).
    pub i18n_config: Option<I18nConfig>,
}

impl<'a> Crate<'a> {
    /// Read crate from `Cargo.toml` i18n config using the
    /// `config_file_path` (if there is one).
    pub fn from(
        path: Box<Path>,
        parent: Option<&'a Crate>,
        config_file_path: Box<Path>,
    ) -> Result<Crate<'a>> {
        let cargo_path = path.join("Cargo.toml");
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

        let full_config_file_path = path.join(&config_file_path);
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
            path,
            parent,
            config_file_path,
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
                    Some(gettext_config) => match gettext_config.extract_to_parent {
                        Some(extract_to_parent) => {
                            if extract_to_parent {
                                return self.parent_active_config();
                            }
                        }
                        None => {}
                    },
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
}


#[derive(Deserialize, Debug)]
pub struct I18nConfig {
    /// The locale/language identifier of the language used in the source code.
    pub src_locale: String,
    /// The locales that the software will be translated into.
    pub target_locales: Vec<String>,
    /// Specify which subcrates to perform localization within. The
    /// subcrate needs to have its own `i18n.toml`.
    pub subcrates: Option<Vec<Box<Path>>>,
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
    pot_dir: Option<PathBuf>,
    /// Path to where the po files will be stored/edited with the
    /// [run_msgmerge()](run_msgmerge()) and
    /// [run_msginit()](run_msginit()) commands, and where they will
    /// be read from with the [run_msgfmt()](run_msgfmt()) command.
    po_dir: Option<PathBuf>,
    /// Path to where the mo files will be written to by the
    /// [run_msgfmt()](run_msgfmt()) command.
    mo_dir: Option<PathBuf>,
}

impl GettextConfig {
    /// Path to where the pot files will be written to by
    /// [run_xtr()](run_xtr()), and were they will be read from by
    /// [run_msginit()](run_msginit()) and
    /// [run_msgmerge()](run_msgmerge()).
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/pot**.
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
    /// [run_msgmerge()](run_msgmerge()) and
    /// [run_msginit()](run_msginit()) commands, and where they will
    /// be read from with the [run_msgfmt()](run_msgfmt()) command.
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/po**.
    pub fn po_dir(&self) -> PathBuf {
        self.po_dir.clone().unwrap_or(self.output_dir.join("po"))
    }

    /// Path to where the mo files will be written to by the
    /// [run_msgfmt()](run_msgfmt()) command.
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/mo**.
    pub fn mo_dir(&self) -> PathBuf {
        self.mo_dir.clone().unwrap_or(self.output_dir.join("mo"))
    }
}