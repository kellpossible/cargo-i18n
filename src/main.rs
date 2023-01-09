use anyhow::Result;
use clap::{crate_authors, crate_version, App, Arg};
use i18n_build::run;
use i18n_config::Crate;
use i18n_embed::{
    gettext::{gettext_language_loader, GettextLanguageLoader},
    DefaultLocalizer, DesktopLanguageRequester, I18nAssets, LanguageLoader, LanguageRequester,
    Localizer,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use std::{path::Path, sync::Arc};
use tr::tr;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n/mo"]
struct Translations;

lazy_static! {
    static ref LANGUAGE_LOADER: GettextLanguageLoader = gettext_language_loader!();
}

static TRANSLATIONS: Translations = Translations {};

/// Produce the message to be displayed when running `cargo i18n -h`.
fn short_about() -> String {
    // The help message displayed when running `cargo i18n -h`.
    tr!("A Cargo sub-command to extract and build localization resources.")
}

fn available_languages(localizer: &Arc<dyn Localizer>) -> Result<Vec<String>> {
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
https://github.com/rust-locale/locale_config#supported-systems ) \
however you can override it using the -l, --language option.

Logging for this command is available using the \"env_logger\" crate. \
You can enable debug logging using \"RUST_LOG=debug cargo i18n\".",
        short_about()
    )
}

fn main() -> Result<()> {
    env_logger::init();
    let mut language_requester = DesktopLanguageRequester::new();

    let cargo_i18n_localizer =
        DefaultLocalizer::new(&*LANGUAGE_LOADER, &TRANSLATIONS as &dyn I18nAssets);

    let cargo_i18n_localizer_rc: Arc<dyn Localizer> = Arc::new(cargo_i18n_localizer);
    let i18n_build_localizer_rc = Arc::new(i18n_build::localizer()) as Arc<dyn Localizer>;

    language_requester.add_listener(Arc::downgrade(&cargo_i18n_localizer_rc));
    language_requester.add_listener(Arc::downgrade(&i18n_build_localizer_rc));
    language_requester.poll()?;

    let fallback_locale = LANGUAGE_LOADER.fallback_language().to_string();
    let available_languages = available_languages(&cargo_i18n_localizer_rc)?;
    let available_languages_slice: Vec<&str> =
        available_languages.iter().map(|l| l.as_str()).collect();

    let matches = App::new("cargo-i18n")
        .bin_name("cargo")
        .term_width(80)
        .about(
            tr!(
                // The message displayed when running the binary outside of cargo using `cargo-18n`.
                "This binary is designed to be executed as a cargo subcommand using \"cargo i18n\".").as_str()
        )
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(App::new("i18n")
            .about(short_about().as_str())
            .long_about(long_about().as_str())
            .version(crate_version!())
            .author(crate_authors!())
            .arg(Arg::new("path")
                .help(
                    // The help message for the `--path` command line argument.
                    tr!("Path to the crate you want to localize (if not the current directory). The crate needs to contain \"i18n.toml\" in its root.").as_str()
                    )
                .long("path")
                .takes_value(true)
            )
            .arg(Arg::new("config file name")
                .help(
                    tr!(
                        // The help message for the `-c`, `--config-file-name` command line argument.
                        "The name of the i18n config file for this crate").as_str()
                )
                .long("config-file-name")
                .short('c')
                .takes_value(true)
                .default_value("i18n.toml")
            )
            .arg(Arg::new("language")
                .help(
                    tr!(
                        // The help message for the `-l`, `--language` command line argument.
                        "Set the language to use for this application. Overrides the language selected automatically by your operating system."
                    ).as_str()
                )
                .long("--language")
                .short('l')
                .takes_value(true)
                .default_value(&fallback_locale)
                .possible_values(available_languages_slice)
            )
        )
        .get_matches();

    if let Some(i18n_matches) = matches.subcommand_matches("i18n") {
        let config_file_name = i18n_matches
            .value_of("config file name")
            .expect("expected a default config file name to be present");

        let language = i18n_matches
            .value_of("language")
            .expect("expected a default language to be present");

        let li: LanguageIdentifier = language.parse()?;
        language_requester.set_language_override(Some(li))?;
        language_requester.poll()?;

        let path = i18n_matches
            .value_of("path")
            .map(|path_str| Path::new(path_str).to_path_buf())
            .unwrap_or_else(|| Path::new(".").to_path_buf());

        let config_file_path = Path::new(config_file_name).to_path_buf();

        i18n_build::util::check_path_exists(&path)?;
        i18n_build::util::check_path_exists(path.join(&config_file_path))?;

        let crt: Crate = Crate::from(path, None, config_file_path)?;
        run(crt)?;
    }

    Ok(())
}
