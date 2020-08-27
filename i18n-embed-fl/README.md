# i18n-embed-fl [![crates.io badge](https://img.shields.io/crates/v/i18n-embed-fl.svg)](https://crates.io/crates/i18n-embed-fl) [![docs.rs badge](https://docs.rs/i18n-embed-fl/badge.svg)](https://docs.rs/i18n-embed-fl/) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed-fl/LICENSE.txt) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

This crate provides a macro to perform compile time checks when using the [i18n-embed](https://crates.io/crates/i18n-embed) crate and the [fluent](https://www.projectfluent.org/) localization system.

**[Changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-embed-fl/CHANGELOG.md)**

## Example

`i18n.toml`:

```toml
fallback_language = "en-GB"

[fluent]
assets_dir = "i18n"
```

`i18n/en-GB/my_crate.ftl`:

```fluent
hello-arg = Hello {$name}!
```

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
    .load_languages(&Localizations, &[loader.fallback_language()])
    .unwrap();

assert_eq!(
    "Hello \u{2068}Bob 23\u{2069}!",
    // Compile time check for message id, and the `name` argument,
    // to ensure it matches what is specified in the `fallback_language`'s
    // fluent resource file.
    fl!(loader, "hello-arg", name = format!("Bob {}", 23))
)
```

See [docs](https://docs.rs/i18n-embed-fl/), and [i18n-embed](https://crates.io/crates/i18n-embed) for more information.
