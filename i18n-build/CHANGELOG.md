# Changelog for `i18n-build`

## v0.4.0

+ Update to `i18n-embed` version `0.5.0`.
+ Change `localizer()` method to return `DefaultLocalizer` instead of the boxed trait `Box<dyn Localizer<'static>>`.

## v0.3.1

+ Update to `i18n-embed` version `0.4.0`.

## v0.3.0

+ Add support for `xtr` `add-location` option.
+ Requires `xtr` version `0.1.5`.
+ Suppress progress output for `msgmerge` using `--silent`.

## v0.2.1

+ Updated link to this changelog in the crate README.

## v0.2.0

+ Bump `i18n-config` version to `0.2`.
+ Handle the situation correctly where the `run()` is called on a crate which is not the root crate, and which makes use of the `gettext` `extract_to_parent` option. This solves [issue 13](https://github.com/kellpossible/cargo-i18n/issues/13).
+ Altered the signature of the `run()` method to take the `Crate` by value.
