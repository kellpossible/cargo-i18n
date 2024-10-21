use serde::Deserialize;
use std::path::PathBuf;

use crate::Crate;

/// The data structure representing what is stored (and possible to
/// store) within the `gettext` subsection of a `i18n.toml` file.
#[derive(Deserialize, Debug, Clone)]
pub struct GettextConfig {
    /// Override the name of the crate, only required if the crate is a virtual
    /// workspace.
    pub name: Option<String>,
    /// Override the version of the crate, only required if the crate is a
    /// virtual workspace.
    pub version: Option<String>,
    /// The languages that the software will be translated into.
    target_languages: Option<Vec<String>>,
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
    /// Generate ‘#: filename:line’ lines (default) in the pot files when
    /// running the `xtr` command. If the type is ‘full’ (the default),
    /// it generates the lines with both file name and line number.
    /// If it is ‘file’, the line number part is omitted. If it is ‘never’,
    ///  nothing is generated. [possible values: full, file, never].
    #[serde(default)]
    pub add_location: GettextAddLocation,
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
    /// Path to where source files are located to have their messages extracted.
    src_dir: Option<PathBuf>,
    /// Enable the `--use-fuzzy` option for the `msgfmt` command.
    ///
    /// By default this is **false**.
    #[serde(default)]
    pub use_fuzzy: bool,
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
        self.pot_dir
            .clone()
            .unwrap_or_else(|| self.output_dir.join("pot"))
    }

    /// Path to where the po files will be stored/edited with the
    /// `msgmerge` and `msginit` commands, and where they will
    /// be read from with the `msgfmt` command.
    ///
    /// By default this is **[output_dir](GettextConfig::output_dir)/po**.
    pub fn po_dir(&self) -> PathBuf {
        self.po_dir
            .clone()
            .unwrap_or_else(|| self.output_dir.join("po"))
    }

    /// Path to where the mo files will be written to by the `msgfmt` command.
    ///
    /// By default this is
    /// **[output_dir](GettextConfig::output_dir)/mo**.
    pub fn mo_dir(&self) -> PathBuf {
        self.mo_dir
            .clone()
            .unwrap_or_else(|| self.output_dir.join("mo"))
    }

    /// Path to where source files are located to have their messages extracted.
    ///
    /// By default is **`src`**.
    pub fn src_dir(&self) -> PathBuf {
        self.src_dir.clone().unwrap_or_else(|| PathBuf::from("src"))
    }

    pub fn target_languages<'a>(&'a self, crt: &'a Crate) -> Option<Vec<String>> {
        self.target_languages.clone().or(crt
            .find_parent()
            .and_then(|p| p.gettext_config_or_err().ok().map(|c| c.target_languages.clone()))
            .flatten())
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
