use anyhow::Result;
use clap::{builder::PossibleValuesParser, crate_authors, crate_version, Arg, Command};
use i18n_build::run;
use i18n_config::{Crate, I18nCargoMetadata};
use i18n_embed::{
    gettext::{gettext_language_loader, GettextLanguageLoader},
    DefaultLocalizer, DesktopLanguageRequester, LanguageLoader, LanguageRequester, Localizer,
};
use rust_embed::RustEmbed;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};
use tr::tr;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n/mo"]
struct Translations;

static TRANSLATIONS: Translations = Translations {};

fn language_loader() -> &'static GettextLanguageLoader {
    static LANGUAGE_LOADER: OnceLock<GettextLanguageLoader> = OnceLock::new();

    LANGUAGE_LOADER.get_or_init(|| gettext_language_loader!())
}

/// Produce the message to be displayed when running `cargo i18n -h`.
fn short_about() -> String {
    // The help message displayed when running `cargo i18n -h`.
    tr!("A Cargo sub-command to extract and build localization resources.")
}

fn available_languages(localizer: &dyn Localizer) -> Result<Vec<String>> {
    Ok(localizer
        .available_languages()?
        .iter()
        .map(|li| li.to_string())
        .collect())
}

/// Produce the message to be displayed when running `cargo i18n --help`.
fn long_about() -> String {
    tr!(
        // The help message displayed when running `cargo i18n --help`.
        // {0} is the short about message.
        // {1} is a list of available languages. e.g. "en", "ru", etc
        "{0}

This command reads a configuration file (typically called \"i18n.toml\") \
in the root directory of your crate, and then proceeds to extract \
localization resources from your source files, and build them.

If you are using the gettext localization system, you will \
need to have the following gettext tools installed: \"msgcat\", \
\"msginit\", \"msgmerge\" and \"msgfmt\". You will also need to have \
the \"xtr\" tool installed, which can be installed using \"cargo \
install xtr\".

You can the \"i18n-embed\" library to conveniently embed the \
localizations inside your application.

The display language used for this command is selected automatically \
using your system settings (as described at 
https://github.com/1Password/sys-locale?tab=readme-ov-file#sys-locale ) \
however you can override it using the -l, --language option.

Logging for this command is available using the \"env_logger\" crate. \
You can enable debug logging using \"RUST_LOG=debug cargo i18n\".",
        short_about()
    )
}

fn main() -> Result<()> {
    env_logger::init();
    let mut language_requester = DesktopLanguageRequester::new();

    let cargo_i18n_localizer: DefaultLocalizer<'static> =
        DefaultLocalizer::new(language_loader(), &TRANSLATIONS);

    let cargo_i18n_localizer_rc: Arc<dyn Localizer> = Arc::new(cargo_i18n_localizer);
    let i18n_build_localizer_rc: Arc<dyn Localizer> = Arc::new(i18n_build::localizer());

    language_requester.add_listener(Arc::downgrade(&cargo_i18n_localizer_rc));
    language_requester.add_listener(Arc::downgrade(&i18n_build_localizer_rc));
    language_requester.poll()?;

    let fallback_locale: &'static str =
        String::leak(language_loader().fallback_language().to_string());
    let available_languages = available_languages(&*cargo_i18n_localizer_rc)?;
    let available_languages_slice: Vec<&'static str> = available_languages
        .into_iter()
        .map(|l| String::leak(l) as &str)
        .collect();

    let matches = Command::new("cargo-i18n")
        .bin_name("cargo")
        .term_width(80)
        .about(
            tr!(
                // The message displayed when running the binary outside of cargo using `cargo-18n`.
                "This binary is designed to be executed as a cargo subcommand using \"cargo i18n\".")
        )
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(Command::new("i18n")
            .about(short_about())
            .long_about(long_about())
            .version(crate_version!())
            .author(crate_authors!())
            .arg(Arg::new("path")
                .help(
                    // The help message for the `--path` command line argument.
                    tr!("Path to the crate you want to localize (if not the current directory). The crate needs to contain \"i18n.toml\" in its root.")
                    )
                .long("path")
                .num_args(1)
            )
            .arg(Arg::new("config-file-name")
                .help(
                    tr!(
                        // The help message for the `-c`, `--config-file-name` command line argument.
                        "The name of the i18n config file for this crate")
                )
                .long("config-file-name")
                .short('c')
                .num_args(1)
            )
            .arg(Arg::new("language")
                .help(
                    tr!(
                        // The help message for the `-l`, `--language` command line argument.
                        "Set the language to use for this application. Overrides the language selected automatically by your operating system."
                    )
                )
                .long("language")
                .short('l')
                .num_args(1)
                .default_value(fallback_locale)
                .value_parser(PossibleValuesParser::new(available_languages_slice))
            )
        )
        .get_matches();

    if let Some(i18n_matches) = matches.subcommand_matches("i18n") {
        let config_file_name: Option<&String> = i18n_matches.get_one("config-file-name");

        // Attempt to read the package.metadata.cargo-i18n.config-path entry in the cargo manifest.
        // If it is not found, we then fall back to the default i18n.toml.
        let cargo_metadata = I18nCargoMetadata::from_cargo_manifest("./Cargo.toml");
        let cargo_config_file_name = cargo_metadata.ok().and_then(|m| m.config_path);

        if cargo_config_file_name.is_some() && config_file_name.is_some() {
            log::warn!("Config file name specified in CLI arguments and Cargo metadata. Ignoring argument and continuing.");
        }

        let config_file_path = Path::new(
            config_file_name
                .or(cargo_config_file_name.as_ref())
                .map(String::as_str)
                .unwrap_or("i18n.toml"),
        )
        .to_path_buf();

        let language: &String = i18n_matches
            .get_one("language")
            .expect("expected a default language to be present");
        let li: LanguageIdentifier = language.parse()?;

        language_requester.set_language_override(Some(li))?;
        language_requester.poll()?;

        let path = i18n_matches
            .get_one::<PathBuf>("path")
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| PathBuf::from("."));

        i18n_build::util::check_path_exists(&path)?;
        i18n_build::util::check_path_exists(path.join(&config_file_path))?;

        let crt: Crate = Crate::from(path, None, config_file_path)?;
        run(crt)?;
    }

    Ok(())
}
