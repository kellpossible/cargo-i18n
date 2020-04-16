# Changelog for `i18n-config`

## v0.2.1

+ Updated link to this changelog in the crate README.

## v0.2.0

+ A bunch of changes to help with solving [issue 13](https://github.com/kellpossible/cargo-i18n/issues/13).
+ Add some debug logging using the [log crate](https://crates.io/crates/log).
+ Migrate away from `anyhow` and provide a new `I18nConfigError` type.
+ Change `I18nConfig#subcrates` type from `Option<Vec<PathBuf>>` to `Vec<PathBuf>` and use `serde` default of empty vector.
+ Add a `find_parent` method which searches.
