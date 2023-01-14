use i18n_embed::DesktopLanguageRequester;
use library_fluent::{hello_world, localizer};

fn main() {
    let library_localizer = localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = library_localizer.select(&requested_languages) {
        eprintln!("Error while loading languages for library_fluent {error}");
    }

    println!("{}", hello_world());
}
