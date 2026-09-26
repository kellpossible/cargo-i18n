# i18n-embed

[![crates.io badge](https://img.shields.io/crates/v/i18n-embed.svg)](https://crates.io/crates/i18n-embed)
[![docs.rs badge](https://docs.rs/i18n-embed/badge.svg)](https://docs.rs/i18n-embed/)
![rust version badge](https://img.shields.io/badge/rustc-1.85.1+-blue.svg)
[![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed/LICENSE.txt)
[![github actions badge](https://github.com/kellpossible/cargo-i18n/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/kellpossible/cargo-i18n/actions/workflows/rust.yml?branch=master)
[![changelog badge](https://img.shields.io/badge/Changelog-8A2BE2)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed/CHANGELOG.md)

- [Overview](#overview)
- [Optional features](#optional-features)
- [Examples](#examples)
  - [Fluent](#fluent-localization-system)
  - [Gettext](#gettext-localization-system)
  - [Automatically updating the requested language](#automatically-updating-the-requested-language)
- [Localizing libraries](#localizing-libraries)
- [Localizing sub-crates](#localizing-sub-crates)

## Overview

Traits and macros to conveniently embed localization assets into your application binary or library in order to localize it at runtime. Works in unison with [cargo-i18n](https://crates.io/crates/cargo_i18n).

This library recommends that you make use of [rust-embed](https://crates.io/crates/rust-embed) to perform the actual embedding of the language files with the `rust-embed` feature.
Using this feature currently requires you to manually add `rust-embed` as a dependency to your project and derive [`RustEmbed`](rust_embed::RustEmbed) on your struct in addition to [`I18nAssets`](I18nAssets).
`RustEmbed` will not compile if the target folder path is invalid, so it is recommended to either run `cargo i18n` before building your project, or commit the localization assets into source control to ensure that the the folder exists and project can build without requiring `cargo i18n`.

## Optional features

The `i18n-embed` crate has the following optional Cargo features:

- `rust-embed` (Enabled by default)
  - Enables an automatic implementation of [`I18nAssets`](I18nAssets) for any type that also implements [`RustEmbed`](rust_embed::RustEmbed).
- `fluent-system`
  - Enables support for the [fluent](https://www.projectfluent.org/) localization system via [`FluentLanguageLoader`](fluent::FluentLanguageLoader).
- `gettext-system`
  - Enables support for the [gettext](https://www.gnu.org/software/gettext/) localization system using the [tr macro](https://docs.rs/tr) and the [gettext crate](https://docs.rs/gettext) via [`GettextLanguageLoader`](gettext::GettextLanguageLoader).
- `desktop-requester`
  - Enables a convenience implementation of the [`LanguageRequester`](LanguageRequester) trait, [`DesktopLanguageRequester`](DesktopLanguageRequester), for the desktop platform (Windows, Mac, Linux), which makes use of the [sys-locale](https://crates.io/crates/sys-locale) crate for resolving the current system locale.
- `web-sys-requester`
  - Enables a convenience implementation of the [`LanguageRequester`](LanguageRequester) trait, [`WebLanguageRequester`](WebLanguageRequester), which makes use of the [web-sys](https://crates.io/crates/web-sys) crate for resolving the language being requested by the user's web browser in a WASM context.
- `filesystem-assets`
  - Enables [`FileSystemAssets`](assets::FileSystemAssets) for loading assets at runtime from the filesystem.
- `autoreload`
  - Enables [`FileSystemAssets::notify_changes_enabled`](FileSystemAssets::notify_changes_enabled) to toggle whether notifications for changed files are emitted.
  - Enables [`RustEmbedNotifyAssets`], a wrapper for [`RustEmbed`](rust_embed::RustEmbed) that emits notifications when files have changed.

## Examples

The example projects can be found [here](https://github.com/kellpossible/cargo-i18n/tree/master/i18n-embed/examples).

### Fluent localization system

The following is a minimal example for how to localize your binary using this library using the [fluent](https://www.projectfluent.org/) localization system.

The [`FluentLanguageLoader`](fluent::FluentLanguageLoader) in this example is instantiated using the [`fluent_language_loader!()`](fluent::fluent_language_loader) macro, which automatically determines the correct module for the crate, and pulls settings in from the `i18n.toml` configuration file.

Start by adding `i18n-embed` as a dependency with the `fluent-system` and `desktop-requester` features enabled:

```toml
[dependencies]
i18n-embed = { version = "0.16.0", features = ["fluent-system", "desktop-requester"]}
rust-embed = "8"
unic-langid = "0.9"
```

Set up a minimal `i18n.toml` in your crate root to use with `cargo-i18n` (see [cargo-i18n](https://github.com/kellpossible/cargo-i18n/blob/master/README.md#configuration) for more information on the configuration file format):

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

Next, you want to create your localization resources, per language fluent (`.ftl`) files. `language` needs to conform to the [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier) standard, and will be parsed via the [unic_langid crate](https://docs.rs/unic-langid).

The directory structure should look like this:

```txt
my_crate/
  Cargo.toml
  i18n.toml
  src/
  i18n/
    {language}/
      {domain}.ftl
```

Then, in your Rust code, add:

```rust
use i18n_embed::{DesktopLanguageRequester, fluent::{
    FluentLanguageLoader, fluent_language_loader
}};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

fn main() {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(
        &language_loader, &Localizations, &requested_languages);

    // continue on with your application
}
```

To access localizations, you can use `FluentLanguageLoader` directly, or, for added compile-time checks/safety, you can use the [fl!() macro](https://crates.io/crates/i18n-embed-fl).

### Gettext localization system

The following is a simple example of how to localize your binary using this library and the `gettext` localization system.

Please note that `gettext` is technically inferior to `fluent` [in a number of ways](https://github.com/projectfluent/fluent/wiki/Fluent-vs-gettext); however, it may be needed due to legacy software, and the developer/translator ecosystem around `gettext` is mature.

The [`GettextLanguageLoader`](gettext::GettextLanguageLoader) in this example is instantiated using the [`gettext_language_loader!()`](gettext::gettext_language_loader) macro, which automatically determines the correct module for the crate, and pulls settings in from the `i18n.toml` configuration file.

Start by adding `i18n-embed` as a dependency with the `gettext-system` and `desktop-requester` features enabled:

```toml
[dependencies]
i18n-embed = { version = "0.16.0", features = ["gettext-system", "desktop-requester"]}
rust-embed = "8"
unic-langid = "0.9"
```

Set up a minimal `i18n.toml` in your crate root to use with `cargo-i18n` (see [cargo-i18n](https://github.com/kellpossible/cargo-i18n#configuration) for more information on the configuration file format):

```toml
# (Required) The language identifier of the language used in the
# source code for gettext system, and the primary fallback language
# (for which all strings must be present) when using the fluent
# system.
fallback_language = "en"

# Use the gettext localization system.
[gettext]
# (Required) The languages that the software will be translated into.
target_languages = ["es"]

# (Required) Path to the output directory, relative to `i18n.toml` of
# the crate being localized.
output_dir = "i18n"
```

Install and run [cargo-i18n](https://crates.io/crates/cargo-i18n) for your crate to generate the language specific `po` and `mo` files, ready to be translated. It is recommended to add the `i18n/pot` folder to your repository `.gitignore`.

Then, in your Rust code, add:

```rust
use i18n_embed::{DesktopLanguageRequester, gettext::{
    gettext_language_loader
}};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
// path to the compiled localization resources,
// as determined by i18n.toml settings
#[folder = "i18n/mo"]
struct Localizations;

fn main() {
    // Create the GettextLanguageLoader, pulling in settings from `i18n.toml`
    // at compile time using the macro.
    let language_loader = gettext_language_loader!();

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let requested_languages = DesktopLanguageRequester::requested_languages();

    let _result = i18n_embed::select(
        &language_loader, &Localizations, &requested_languages);

    // continue on with your application
}
```

### Automatically updating the requested language

Depending on the platform, you can also make use of [`LanguageRequester`](`LanguageRequester`)'s ability to monitor changes to the currently requested language, and automatically update the selected language using a [`Localizer`](Localizer):

```rust
use std::sync::{Arc, OnceLock};
use i18n_embed::{
    DesktopLanguageRequester, LanguageRequester,
    DefaultLocalizer, Localizer, fluent::FluentLanguageLoader
};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n/ftl"] // path to localization resources
struct Localizations;

pub fn language_loader() -> &'static FluentLanguageLoader {
    static LANGUAGE_LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();

    LANGUAGE_LOADER.get_or_init(|| {
        // Usually you could use the fluent_language_loader!() macro
        // to pull values from i18n.toml configuration and current
        // module here at compile time, but instantiating the loader
        // manually here instead so the example compiles.
        let fallback: LanguageIdentifier = "en-US".parse().unwrap();
        FluentLanguageLoader::new("test", fallback)
    })
}

fn main() {
    let localizer = DefaultLocalizer::new(&*language_loader(), &Localizations);

    let localizer_arc: Arc<dyn Localizer> = Arc::new(localizer);

    let mut language_requester = DesktopLanguageRequester::new();
    language_requester.add_listener(Arc::downgrade(&localizer_arc));

    // Manually check the currently requested system language,
    // and update the listeners. NOTE: Support for this across systems
    // currently varies. It may not change when the system requested
    // language changes during runtime without restarting your application.
    // In the future some platforms may also gain support for
    // automatic triggering when the requested display language changes.
    language_requester.poll().unwrap();

    // continue on with your application
}
```

The above example makes use of the [`DefaultLocalizer`](DefaultLocalizer) implementation, but you can also implement the [`Localizer`](Localizer) trait yourself for a custom solution.

## Localizing libraries

If you wish to create a localizable library using `i18n-embed`, you can follow this pattern in the library:

```rust
use std::sync::{Arc, OnceLock};
use i18n_embed::{
    DefaultLocalizer, Localizer, LanguageLoader,
    fluent::{
        fluent_language_loader, FluentLanguageLoader
}};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/mo"] // path to the compiled localization resources
struct Localizations;

fn language_loader() -> &'static FluentLanguageLoader {
    static LANGUAGE_LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();

    LANGUAGE_LOADER.get_or_init(|| {
       let loader = fluent_language_loader!();

        // Load the fallback language by default so that users of the
        // library don't need to if they don't care about localization.
        // This isn't required for the `gettext` localization system.
        loader.load_fallback_language(&Localizations)
            .expect("Error while loading fallback language");

        loader
    })
}

/// Get the `Localizer` to be used for localizing this library.
pub fn localizer() -> Arc<dyn Localizer> {
    Arc::new(DefaultLocalizer::new(&*language_loader(), &Localizations))
}
```

People using this library can call `localize()` to obtain a [`Localizer`](Localizer), and add this as a listener to their chosen [`LanguageRequester`](LanguageRequester).

## Localizing sub-crates

If you want to localize a sub-crate in your project, and want to extract strings from this sub-crate and store/embed them in one location in the parent crate, you can use the following pattern for the library:

```rust
use std::sync::{Arc, OnceLock};
use i18n_embed::{
DefaultLocalizer, Localizer, gettext::{
  gettext_language_loader, GettextLanguageLoader
}};
use i18n_embed::I18nAssets;

fn language_loader() -> &'static GettextLanguageLoader {
  static LANGUAGE_LOADER: OnceLock<GettextLanguageLoader> = OnceLock::new();
  LANGUAGE_LOADER.get_or_init(|| gettext_language_loader!())
}

/// Get the `Localizer` to be used for localizing this library,
/// using the provided embedded source of language files `embed`.
pub fn localizer<'a>(embed: &'a (dyn I18nAssets + Send + Sync + 'static)) -> Arc<dyn Localizer + 'a> {
  Arc::new(DefaultLocalizer::new(language_loader(), embed))
}
```

For the above example, you can enable the following options in the sub-crate's `i18n.toml` to ensure that the localization resources are extracted and merged with the parent crate's `pot` file:

```toml
# ...
[gettext]
# ...
# (Optional) If this crate is being localized as a subcrate, store the final
# localization artifacts (the module pot and mo files) with the parent crate's
# output. Currently crates which contain subcrates with duplicate names are not
# supported.
extract_to_parent = true
# (Optional) If a subcrate has extract_to_parent set to true, then merge the
# output pot file of that subcrate into this crate's pot file.
collate_extracted_subcrates = true
```
