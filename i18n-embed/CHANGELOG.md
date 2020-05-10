# Changelog for `i18n-embed`

## v0.4.2

+ Update `fluent-langneg` dependency to version `0.13`.
+ Update `unic-langid` dependency to version `0.9`.

## v0.4.1

+ Fix incorrect comment in code example [#18](https://github.com/kellpossible/cargo-i18n/issues/18).

## v0.4.0

Mostly a refactor of `LanguageLoader` and `I18nEmbed` to solve [issue #15](https://github.com/kellpossible/cargo-i18n/issues/15).

+ Replaced the derive macro for `LanguageLoader` with a new `language_loader!(StructName)` which creates a new struct with the specified `StructName` and implements `LanguageLoader` for it. This was done because `LanguageLoader` now needs to store state for the currently selected language, and deriving this automatically would be complicated.
+ Refactored `I18nEmbed` to move the `load_language_file` responsibility into `LanguageLoader` and add a new `load_language` method to `LanguageLoader`.
+ Refactored `I18nEmbedDyn` to also expose the `RustEmbed#get()` method, required for the new `LanguageLoader` changes.
+ Using `LanguageLoader` as a static now requires [lazy_static](https://crates.io/crates/lazy_static) or something similar because the `StructName#new()` constructor which is created for it in `language_loader!(StructName)` is not `const`.

## v0.3.4

+ Made `WebLanguageRequester::requested_languages()` public.

## v0.3.3

+ Updated link to this changelog in the crate README.

## v0.3.2

+ Bump `i18n-config` dependency in `i18n-embed-impl` version to `0.2`.
