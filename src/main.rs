use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
/// This binary is designed to be executed as a cargo subcommand using
/// `cargo i18n`.
enum Opt {
    #[structopt(name = "i18n")]
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
    I18n {
        #[structopt(long)]
        /// Path to the crate you want to localize (if not the current
        /// directory). The crate to contain `i18n.toml` in its root.
        path: Option<String>,
    },
}

fn main() {
    let opt = Opt::from_args();

    println!("Hello, world!");
}
