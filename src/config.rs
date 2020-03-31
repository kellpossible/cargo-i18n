use crate::gettext::GettextConfig;

use std::fs::read_to_string;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_derive::Deserialize;
use toml;
use tr::tr;

pub struct Crate {
    pub name: String,
    pub path: Box<Path>,
}

impl Crate {
    pub fn new<S: Into<String>>(name: S, path: Box<Path>) -> Crate {
        Crate {
            name: name.into(),
            path,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct I18nConfig {
    /// The locale/language identifier of the language used in the source code.
    pub src_locale: String,
    /// The locales that the software will be translated into.
    pub locales: Vec<String>,
    /// Path to the output directory, relative to `i18n.toml` of the
    /// crate being localized.
    pub output_dir: Box<Path>,
    /// Specify which subcrates to perform localization within. If the
    /// subcrate has its own `i18n.toml` then, it will have its
    /// localization performed independently (rather than being
    /// incorporated into the parent project's localization).
    pub subcrates: Option<Vec<Box<Path>>>,
    /// The subcomponent of this config relating to gettext, only
    /// present if the gettext localization system will be used.
    pub gettext_config: Option<GettextConfig>,
}

impl I18nConfig {
    pub fn from_file(toml_path: &Path) -> Result<I18nConfig> {
        let toml_str = read_to_string(toml_path).context("trouble reading i18n.toml")?;
        let config: I18nConfig =
            toml::from_str(toml_str.as_ref()).context("trouble parsing i18n.toml")?;
        Ok(config)
    }

    pub fn gettext_config(&self) -> Result<&GettextConfig> {
        match &self.gettext_config {
            Some(gettext_config) => Ok(gettext_config),
            None => Err(anyhow!(tr!(
                "there is no gettext config available in this i18n config"
            ))),
        }
    }
}
