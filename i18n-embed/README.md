# i18n-embed [![crates.io badge](https://img.shields.io/crates/v/i18n-embed.svg)](https://crates.io/crates/i18n-embed) [![docs.rs badge](https://docs.rs/i18n-embed/badge.svg)](https://docs.rs/i18n-embed/) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-build/LICENSE.txt) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

This library contains traits and macros to conveniently embed the output of [cargo-i18n](https://crates.io/crates/cargo_i18n) into your application binary in order to localize it at runtime.

Currently this library depends on [rust-embed](https://crates.io/crates/rust-embed) to perform the actual embedding of the language files. This may change in the future to make the library more convenient to use.

**[Changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed/CHANGELOG.md)**

## Optional Features

The `i18n-embed` crate has the following optional Cargo features:

+ `desktop-requester`
  + Enables a convenience implementation of `LanguageRequester` trait called `DesktopLanguageRequester` for the desktop platform (windows, mac, linux),which makes use of the [locale_config](https://crates.io/crates/locale_config) crate for resolving the current system locale.
+ `web-sys-requester`
  + Enables a convenience implementation of `LanguageRequester` trait called `WebLanguageRequester` which makes use of the [web-sys](https://crates.io/crates/web-sys) crate for resolving the language being requested by the user's web browser in a WASM context.

## Example

The following is an example for how to derive the required traits on structs, and localize your binary using this library:

```rust
use i18n_embed::{I18nEmbed, language_loader, DesktopLanguageRequester};
use rust_embed::RustEmbed;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"] // path to the compiled localization resources
struct Translations;

language_loader!(MyLanguageLoader);

fn main() {
    let translations = Translations {};
    let language_loader = MyLanguageLoader::new();

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let requested_languages = DesktopLanguageRequester::requested_languages();

    i18n_embed::select(&language_loader, &translations, &requested_languages);
}
```

For more examples, see the [documentation for i18n-embed](https://docs.rs/i18n-embed/).
