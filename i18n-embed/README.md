# i18n-embed [![crates.io badge](https://img.shields.io/crates/v/i18n-embed.svg)](https://crates.io/crates/i18n-embed) [![docs.rs badge](https://docs.rs/i18n-embed/badge.svg)](https://docs.rs/i18n-embed/) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed/LICENSE.txt) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

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
i18n-embed = { version = "0.8", features = ["fluent-system", "desktop-requester"]}
rust-embed = "5"
unic-langid = "0.9"
```

Set up a minimal `i18n.toml` in your crate root to use with `cargo-i18n` (see [cargo i18n](../README.md#configuration) for more information on the configuration file format):

```toml
# (Required) The language identifier of the language used in the
# source code for gettext system, and the primary fallback language
# (for which all strings must be present) when using the fluent
# system.
fallback_language = "en-GB"

# Use the fluent localization system.
[fluent]
# (Required) The path to the assets directory.
# The paths inside the assets directory should be structured like so:
# `assets_dir/{language}/{domain}.ftl`
assets_dir = "i18n"
```

Next, you want to create your localization resources, per language fluent files. `language` needs to conform to the [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier) standard, and will be parsed via the [unic_langid crate](https://docs.rs/unic-langid/0.9.0/unic_langid/).

The directory structure should look like so:

```txt
my_crate/
  Cargo.toml
  i18n.toml
  src/
  i18n/
    {language}/
      {domain}.ftl
```

Then you can instantiate your language loader and language requester:

```rust
use i18n_embed::{DesktopLanguageRequester, fluent::{
    FluentLanguageLoader, fluent_language_loader
}};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Translations;

fn main() {
    let translations = Translations {};
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(
        &language_loader, &translations, &requested_languages);

    // continue on with your application
}
```

To access localizations, you can use `FluentLanguageLoader`'s methods directly, or, for added compile-time checks/safety, you can use the [fl!() macro](https://crates.io/crates/i18n-embed-fl). Having an `i18n.toml` configuration file enables you to do the following:

+ Use the [cargo i18n](https://crates.io/crates/cargo-i18n) tool   to perform validity checks (not yet implemented).
+ Integrate with a code-base using the `gettext` localization   system.
+ Use the `fluent::fluent_language_loader!()` macro to pull the   configuration in at compile time to create the `fluent::FluentLanguageLoader`.
+ Use the [fl!() macro](https://crates.io/crates/i18n-embed-fl) to have added compile-time safety when accessing messages.

For more examples, see the [documentation for i18n-embed](https://docs.rs/i18n-embed/).
