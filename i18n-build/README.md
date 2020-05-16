# i18n-build [![crates.io badge](https://img.shields.io/crates/v/i18n-build.svg)](https://crates.io/crates/i18n-build) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-build/LICENSE.txt) [![docs.rs badge](https://docs.rs/i18n-build/badge.svg)](https://docs.rs/i18n-build/) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

A library for use within the [cargo-i18n](https://crates.io/crates/cargo_i18n) tool for localizing crates. It has been published to allow its direct use within project build scripts if required.

**[Changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-build/CHANGELOG.md)**

## Optional Features

The `i18n-build` crate has the following optional Cargo features:

+ `localize`
  + Enables the runtime localization of this library using `localize()` function via the [i18n-embed](https://crates.io/crates/i18n-embed) crate.

## Contributing

Pull-requests are welcome, but for design changes it is preferred that you create a GitHub issue first to discuss it before implementation. You can also contribute to the localization of this library via [POEditor - i18n-build](https://poeditor.com/join/project/BCW39cVoco) or use your favourite `po` editor.

To add a new language, you can make a request via a GitHub issue, or submit a pull request adding the new locale to [i18n.toml](https://github.com/kellpossible/cargo-i18n/blob/master/i18n.toml) and generating the associated new po files using `cargo i18n`.

## Authors

+ [Contributors](https://github.com/kellpossible/cargo-i18n/graphs/contributors)
+ [Translators](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-build/i18n/TRANSLATORS)