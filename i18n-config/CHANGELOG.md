# Changelog for `i18n-config`

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
