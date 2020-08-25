# i18n-embed [![crates.io badge](https://img.shields.io/crates/v/i18n-embed.svg)](https://crates.io/crates/i18n-embed) [![docs.rs badge](https://docs.rs/i18n-embed/badge.svg)](https://docs.rs/i18n-embed/) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-build/LICENSE.txt) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

This library contains traits and macros to conveniently embed the output of [cargo-i18n](https://crates.io/crates/cargo_i18n) into your application binary in order to localize it at runtime.

Currently this library depends on [rust-embed](https://crates.io/crates/rust-embed) to perform the actual embedding of the language files. This may change in the future to make the library more convenient to use.

**[Changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed/CHANGELOG.md)**

## Optional Features

The `i18n-embed` crate has the following optional Cargo features:

+ `fluent-system`
  + Enable support for the [fluent](https://www.projectfluent.org/) localization system via the `FluentLanguageLoader`.
+ `gettext-system`
  + Enable support for the [gettext](https://www.gnu.org/software/gettext/) localization system using the [tr macro](https://docs.rs/tr/0.1.3/tr/) and the [gettext crate](https://docs.rs/gettext/0.4.0/gettext/) via the `GettextLanguageLoader`.
+ `desktop-requester`
  + Enables a convenience implementation of `LanguageRequester` trait called `DesktopLanguageRequester` for the desktop platform (windows, mac, linux),which makes use of the [locale_config](https://crates.io/crates/locale_config) crate for resolving the current system locale.
+ `web-sys-requester`
  + Enables a convenience implementation of `LanguageRequester` trait called `WebLanguageRequester` which makes use of the [web-sys](https://crates.io/crates/web-sys) crate for resolving the language being requested by the user's web browser in a WASM context.

## Example

The following is a minimal example for how localize your binary using this
library using the [fluent](https://www.projectfluent.org/) localization system.

First you need to compile `i18n-embed` in your `Cargo.toml` with the `fluent-system` and `desktop-requester` features enabled:

```toml
[dependencies]
i18n-embed = { version = "0.7", features = ["fluent-system", "desktop-requester"]}
rust-embed = "5"
unic-langid = "0.9"
```

Next, you want to create your localization resources, per language fluent files. `lang_code` needs to conform to the [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier) standard, and will be parsed via the [unic_langid crate](https://docs.rs/unic-langid/0.9.0/unic_langid/):

```txt
my_crate/
  Cargo.toml
  i18n.toml
  src/
  i18n/
    lang_code/
      my_crate.ftl
```

Then you can instantiate your language loader and language requester:

```rust
use i18n_embed::{I18nEmbed, DesktopLanguageRequester, fluent::{
  FluentLanguageLoader
}};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n"] // path to the localization resources
struct Translations;

fn main() {
    let translations = Translations {};

    let fallback_language: LanguageIdentifier = "en-US".parse().unwrap();
     let language_loader = FluentLanguageLoader::new(
       "my_crate", fallback_language);

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let requested_languages = DesktopLanguageRequester::requested_languages();

    let _result = i18n_embed::select(
      &language_loader, &translations, &requested_languages);
}
```

You can also make use of the `i18n.toml` configuration file, and the [cargo i18n](https://crates.io/crates/cargo-i18n) tool to integrate with a code-base using `gettext`, and in the future to perform compile time checks, and use the `fluent_language_loader!()` macro to pull the configuration at compile time to create the `FluentLanguageLoader`.

For more examples, see the [documentation for i18n-embed](https://docs.rs/i18n-embed/).
