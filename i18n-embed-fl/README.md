# i18n-embed-fl [![crates.io badge](https://img.shields.io/crates/v/i18n-embed-fl.svg)](https://crates.io/crates/i18n-embed-fl) [![docs.rs badge](https://docs.rs/i18n-embed-fl/badge.svg)](https://docs.rs/i18n-embed-fl/) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed-fl/LICENSE.txt) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

This crate provides a macro to perform compile time checks when using the [i18n-embed](https://crates.io/crates/i18n-embed) crate and the [fluent](https://www.projectfluent.org/) localization system.

See [docs](https://docs.rs/i18n-embed-fl/), and [i18n-embed](https://crates.io/crates/i18n-embed) for more information.

**[Changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed-fl/CHANGELOG.md)**

## Example

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

Create a fluent localization file for the `en-GB` language in `i18n/en-GB/{domain}.ftl`, where `domain` is the rust path of your crate (`_` instead of `-`):

```fluent
hello-arg = Hello {$name}!
```

Simple set up of the `FluentLanguageLoader`, and obtaining a message formatted with an argument:

```rust
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    LanguageLoader,
};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

let loader: FluentLanguageLoader = fluent_language_loader!();
loader
    .load_languages(&Localizations, &[loader.fallback_language().clone()])
    .unwrap();

assert_eq!(
    "Hello \u{2068}Bob 23\u{2069}!",
    // Compile time check for message id, and the `name` argument,
    // to ensure it matches what is specified in the `fallback_language`'s
    // fluent resource file.
    fl!(loader, "hello-arg", name = format!("Bob {}", 23))
)
```

## Convenience Macro

You will notice that this macro requires `loader` to be specified in every call. For you project you may have access to a statically defined loader, and you can create a convenience macro wrapper so this doesn't need to be imported and specified every time.

```rust
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::YOUR_STATIC_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::YOUR_STATIC_LOADER, $message_id, $($args), *)
    }};
}
```

This can now be invoked like so: `fl!("message-id")`, `fl!("message-id", args)` and `fl!("message-id", arg = "value")`.
