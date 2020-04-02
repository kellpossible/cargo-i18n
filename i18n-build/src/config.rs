use crate::gettext_impl::GettextConfig;

use std::fs::read_to_string;
use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, Context, Result};
use serde_derive::Deserialize;
use toml;
use tr::tr;

/// Represents a rust crate
pub struct Crate {
    /// The name of the crate
    pub name: String,
    /// The version of the crate
    pub version: String,
    /// The path to the crate
    pub path: Box<Path>,
    /// Path to the parent crate which is triggering the localization
    /// for this crate.
    pub parent: Option<Rc<Crate>>,
}

impl Crate {
    /// Read crate from `Cargo.toml`
    pub fn from(path: Box<Path>, parent: Option<Rc<Crate>>) -> Result<Crate> {
        let cargo_path = path.join("Cargo.toml");
        let toml_str = read_to_string(cargo_path.clone())
            .with_context(|| format!("trouble reading {0:?}", cargo_path))?;
        let cargo_toml: toml::Value = toml::from_str(toml_str.as_ref())
            .with_context(|| format!("trouble parsing {0:?}", cargo_path))?;

        let package = cargo_toml
            .as_table()
            .ok_or(anyhow!("expected Cargo.toml to be a table"))?
            .get("package")
            .ok_or(anyhow!("expected Cargo.toml to have a `package` section"))?
            .as_table()
            .ok_or(anyhow!(
                "expected Cargo.toml's package section to be a map containing values"
            ))?;

        let name = package
            .get("name")
            .ok_or(anyhow!("expected Cargo.toml's package name to exist"))?
            .as_str()
            .ok_or(anyhow!("expected Cargo.toml'spackage name to be a string"))?;

        let version = package
            .get("version")
            .ok_or(anyhow!("expected Cargo.toml's package version to exist"))?
            .as_str()
            .ok_or(anyhow!("expected Cargo.toml'spackage version to be a string"))?;

        Ok(Crate {
            name: String::from(name),
            version: String::from(version),
            path,
            parent,
        })
    }

    pub fn module_name(&self) -> String {
        self.name.replace("-", "_")
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
        let toml_str = read_to_string(toml_path).context("trouble reading i18n.toml")?;
        let config: I18nConfig =
            toml::from_str(toml_str.as_ref()).context("trouble parsing i18n.toml")?;
        Ok(config)
    }

    pub fn gettext_config(&self) -> Result<&GettextConfig> {
        match &self.gettext {
            Some(gettext_config) => Ok(gettext_config),
            None => Err(anyhow!(tr!(
                "there is no gettext config available in this i18n config"
            ))),
        }
    }
}
