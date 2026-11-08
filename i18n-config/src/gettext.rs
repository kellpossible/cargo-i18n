use serde::{
    Deserialize, Deserializer,
    de::{self, IgnoredAny, Visitor},
};
use std::{fmt, path::PathBuf};

/// The data structure representing what is stored (and possible to
/// store) within the `gettext` subsection of a `i18n.toml` file.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct GettextConfig {
    /// The languages that the software will be translated into.
    pub target_languages: Vec<String>,
    /// Where the localization artifacts from this crate should be stored
    pub output: OutputConfig,
    /// If a subcrate has output.extract_to_parent set to true,
    /// then merge the output pot file of that subcrate into this
    /// crate's pot file.
    ///
    /// By default this is **false**.
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
    /// Generate ‘#: filename:line’ lines (default) in the pot files when
    /// running the `xtr` command. If the type is ‘full’ (the default),
    /// it generates the lines with both file name and line number.
    /// If it is ‘file’, the line number part is omitted. If it is ‘never’,
    ///  nothing is generated. [possible values: full, file, never].
    #[serde(default)]
    pub add_location: GettextAddLocation,
    /// Enable the `--use-fuzzy` option for the `msgfmt` command.
    ///
    /// By default this is **false**.
    #[serde(default)]
    pub use_fuzzy: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum OutputConfig {
    /// Path to the output directory, relative to `i18n.toml` of the
    /// crate being localized.
    Dir {
        dir: PathBuf,
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
    },
    /// If this crate is being localized as a subcrate, store the
    /// localization artifacts with the parent crate's output.
    ///
    /// Currently crates which contain subcrates with duplicate names
    /// are not supported.
    #[serde(deserialize_with = "deserialize_parent")]
    Parent,
}

/// Parses `output.extract_to_parent = true`
struct ParentVisitor;

impl<'v> Visitor<'v> for ParentVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a boolean")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'v>,
    {
        let (name, value) = map
            .next_entry::<String, bool>()?
            .ok_or(de::Error::missing_field("extract_to_parent"))?;
        if name != "extract_to_parent" {
            Err(de::Error::unknown_field(&name, &["extract_to_parent"]))
        } else if !value {
            Err(de::Error::invalid_value(
                de::Unexpected::Bool(value),
                &"`true`",
            ))
        } else if let Some((field, _value)) = map.next_entry::<String, IgnoredAny>()? {
            Err(de::Error::unknown_field(&field, &["extract_to_parent"]))
        } else {
            Ok(())
        }
    }
}

fn deserialize_parent<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(ParentVisitor)
}

impl GettextConfig {
    /// Path to where the pot files will be written to by the `xtr`
    /// command, and were they will be read from by `msginit` and
    /// `msgmerge`.
    ///
    /// By default this is `output.dir/pot`.
    pub fn pot_dir(&self) -> Option<PathBuf> {
        if let OutputConfig::Dir {
            dir,
            pot_dir,
            po_dir: _,
            mo_dir: _,
        } = &self.output
        {
            Some(pot_dir.clone().unwrap_or_else(|| dir.join("pot")))
        } else {
            None
        }
    }

    /// Path to where the po files will be stored/edited with the
    /// `msgmerge` and `msginit` commands, and where they will
    /// be read from with the `msgfmt` command.
    ///
    /// By default this is `output.dir/po`.
    pub fn po_dir(&self) -> Option<PathBuf> {
        if let OutputConfig::Dir {
            dir,
            pot_dir: _,
            po_dir,
            mo_dir: _,
        } = &self.output
        {
            Some(po_dir.clone().unwrap_or_else(|| dir.join("po")))
        } else {
            None
        }
    }

    /// Path to where the mo files will be written to by the `msgfmt` command.
    ///
    /// By default this is `output.dir/mo`.
    pub fn mo_dir(&self) -> Option<PathBuf> {
        if let OutputConfig::Dir {
            dir,
            pot_dir: _,
            po_dir: _,
            mo_dir,
        } = &self.output
        {
            Some(mo_dir.clone().unwrap_or_else(|| dir.join("mo")))
        } else {
            None
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum GettextAddLocation {
    #[default]
    Full,
    File,
    Never,
}

impl GettextAddLocation {
    pub fn to_str(&self) -> &str {
        match self {
            GettextAddLocation::Full => "full",
            GettextAddLocation::File => "file",
            GettextAddLocation::Never => "never",
        }
    }
}
