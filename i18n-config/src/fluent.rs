use serde::Deserialize;
use std::path::PathBuf;

/// The data structure representing what is stored (and possible to
/// store) within the `fluent` subsection of a `i18n.toml` file.
#[derive(Deserialize, Debug, Clone)]
pub struct FluentConfig {
    /// (Required) The path to the assets directory.
    ///
    /// The paths inside the assets directory should be  structured
    /// like so: `assets_dir/{language}/{domain}.ftl`
    pub assets_dir: PathBuf,
}
