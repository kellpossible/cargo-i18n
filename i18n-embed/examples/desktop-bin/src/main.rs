use std::time::Duration;

use i18n_embed::{
    fluent::FluentLanguageLoader, DefaultLocalizer, DesktopLanguageRequester, Localizer,
    RustEmbedNotifyAssets,
};
use library_fluent::{hello_world, localizer, LocalizationsEmbed};

fn main() {
    env_logger::init();
    let library_localizer = localizer().with_autoreload().unwrap();
    let requested_languages = DesktopLanguageRequester::<
        DefaultLocalizer<RustEmbedNotifyAssets<LocalizationsEmbed>, FluentLanguageLoader>,
    >::requested_languages();

    if let Err(error) = library_localizer.select(&requested_languages) {
        eprintln!("Error while loading languages for library_fluent {error}");
    }

    loop {
        println!("{}", hello_world());
        std::thread::sleep(Duration::from_secs(1));
    }
}
