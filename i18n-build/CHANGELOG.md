# Changelog for `i18n-build`

## v0.2.0

+ Bump `i18n-config` version to `0.2`.
+ Handle the situation correctly where the `run()` is called on a crate which is not the root crate, and which makes use of the `gettext` `extract_to_parent` option. This solves [issue 13](https://github.com/kellpossible/cargo-i18n/issues/13).
+ Altered the signature of the `run()` method to take the `Crate` by value.
