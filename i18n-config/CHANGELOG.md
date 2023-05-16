# `i18n-config` Changelog

## master

### New Features

+ Add option to override the default domain name for fluent assets.

### Internal

+ Fix clippy warnings.
+ Bump `toml` to version `0.7`.

## v0.4.3

+ Prevent a panic when the crate root is a workspace. [#93](https://github.com/kellpossible/cargo-i18n/pull/93) thanks to [@ctrlcctrlv](https://github.com/ctrlcctrlv).

## v0.4.2

### New Features

+ Add the `use_fuzzy` option for the `gettext` system. [#68](https://github.com/kellpossible/cargo-i18n/pull/68) thanks to [@vkill](https://github.com/vkill).

## v0.4.1

### New Features

+ New `locate_crate_paths()` function for use in procedural macros.

## v0.4.0

### New Features

+ Introduced new `assets_dir` member of `[fluent]` subsection.

### Breaking Changes

+ Changed type of `fallback_language` from `String` to `unic_langid::LanguageIdentifier`.

### Internal Changes

+ Improved error messages.

## v0.3.0

Changes for the support of the `fluent` localization system.

### New Features

+ New `FluentConfig` (along with associated `[fluent]` subsection in the configuration file format) for using the `fluent` localization system.

### Breaking Changes

+ Renamed `src_locale` to `fallback_language`.
+ Moved `target_locales` to within the `[gettext]` subsection, and renamed it to `target_languages`.

### Internal Changes

+ Now using `parking_lot::RwLock` for the language loaders, instead of the `RwLock` in the standard library.

## v0.2.2

+ Add support for `xtr` `add-location` option.

## v0.2.1

+ Updated link to this changelog in the crate README.

## v0.2.0

+ A bunch of changes to help with solving [issue 13](https://github.com/kellpossible/cargo-i18n/issues/13).
+ Add some debug logging using the [log crate](https://crates.io/crates/log).
+ Migrate away from `anyhow` and provide a new `I18nConfigError` type.
+ Change `I18nConfig#subcrates` type from `Option<Vec<PathBuf>>` to `Vec<PathBuf>` and use `serde` default of empty vector.
+ Add a `find_parent` method which searches.
