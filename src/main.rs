use anyhow::{Context, Result};
use cargo_i18n_lib::config::I18nConfig;
use cargo_i18n_lib::run;
use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use tr::tr;
use i18n_embed::i18n_embed;

i18n_embed!();

fn short_about() -> String {
    tr!("A Cargo subcommand to extract and build localization resources.")
}

fn long_about() -> String {
    tr!(
        "{0}

This command reads the \"i18n.toml\" config in your crate root, \
and based on the configuration there, proceeds to extract \
localization resources, and build them.

If you are using the gettext localization system, you will \
need to have the following gettext tools installed: \"msgcat\", \
\"msginit\", \"msgmerge\" and \"msgfmt\". You will also need to have \
the \"xtr\" tool installed, which can be installed using \"cargo \
install xtr\".
",
        short_about()
    )
}

fn main() -> Result<()> {
    let matches = App::new("cargo-i18n")
        .bin_name("cargo")
        .about(tr!("This binary is designed to be executed as a cargo subcommand using \"cargo i18n\".").as_str())
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(SubCommand::with_name("i18n")
            .about(short_about().as_str())
            .long_about(long_about().as_str())
            .version(crate_version!())
            .author(crate_authors!())
            .arg(Arg::with_name("path")
                .help(tr!(
                    "Path to the crate you want to localize (if not the current directory). The crate needs to contain \"i18n.toml\" in its root.").as_str())
                .long("path")
                .takes_value(true)
            )
            .arg(Arg::with_name("config file name")
                .help(tr!("The name of the i18n config file for this crate").as_str())
                .long("config-file-name")
                .short("c")
                .takes_value(true)
                .default_value("i18n.toml")
            )
        )
        .get_matches();

    if let Some(i18n_matches) = matches.subcommand_matches("i18n") {
        let config_file_name = i18n_matches
            .value_of("config file name")
            .expect("expected a config file name to be present");
        let config = I18nConfig::from_file(config_file_name.clone())
            .with_context(|| tr!("cannot load config file \"{0}\"", config_file_name))?;

        run(&config)?;
    }

    Ok(())
}
