use anyhow::Context;
use cargo_i18n::config::I18nConfig;
use cargo_i18n::run;
use structopt::StructOpt;

/// Cargo subcommand to extract and build localization resources.
///
/// This command reads the `i18n.toml` config in your crate root,
/// and based on the configuration there, proceeds to extract
/// localization resources, and build them.
///
/// If you are using the gettext localization system, you will
/// need to have the following gettext tools installed: `msgcat`,
/// `msginit`, `msgmerge` and `msgfmt`. You will also need to have
/// the `xtr` tool installed, which can be installed using `cargo
/// install xtr`.
#[derive(StructOpt)]
struct I18n {
    #[structopt(long)]
    /// Path to the crate you want to localize (if not the current
    /// directory). The crate to contain `i18n.toml` in its root.
    path: Option<String>,

    #[structopt(long = "config", short = "c", default_value = "i18n.toml")]
    /// The
    config_file: String,
}

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
/// This binary is designed to be executed as a cargo subcommand using
/// `cargo i18n`.
enum Opt {
    #[structopt(name = "i18n")]
    I18n(I18n),
}

fn main() {
    let opt = Opt::from_args();

    let config = match opt {
        Opt::I18n(i18n) => I18nConfig::from_file(i18n.config_file),
    }
    .context("cannot load i18n config file")
    .unwrap();

    run(&config).unwrap();
}
