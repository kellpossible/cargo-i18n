use crate::gettext_impl::GettextConfig;

use std::fs::read_to_string;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_derive::Deserialize;
use toml;
use tr::tr;

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
    /// Read crate from `Cargo.toml`
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
                    tr!(
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

    pub fn module_name(&self) -> String {
        self.name.replace("-", "_")
    }

    pub fn parent_active_config(&'a self) -> Option<(&'a Crate, &'a I18nConfig)> {
        match self.parent {
            Some(parent) => parent.active_config(),
            None => None,
        }
    }

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

    pub fn config_or_err(&self) -> Result<&I18nConfig> {
        match &self.i18n_config {
            Some(config) => Ok(config),
            None => Err(anyhow!(tr!(
                "There is no i18n config called \"{0}\" present in this crate \"{1}\".",
                self.config_file_path.to_string_lossy(),
                self.name
            ))),
        }
    }

    pub fn gettext_config_or_err(&self) -> Result<&GettextConfig> {
        match &self.config_or_err()?.gettext {
            Some(gettext_config) => Ok(gettext_config),
            None => Err(anyhow!(tr!(
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
    pub fn from_file<P: AsRef<Path>>(toml_path: P) -> Result<I18nConfig> {
        let toml_path_final: &Path = toml_path.as_ref();
        let toml_str = read_to_string(toml_path_final).with_context(|| {
            tr!(
                "Trouble reading file \"{0}\".",
                toml_path_final.to_string_lossy()
            )
        })?;
        let config: I18nConfig = toml::from_str(toml_str.as_ref()).with_context(|| {
            tr!(
                "There was an error while parsing an i18n config file \"{0}\".",
                toml_path_final.to_string_lossy()
            )
        })?;
        Ok(config)
    }
}
