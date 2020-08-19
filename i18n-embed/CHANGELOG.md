# Changelog for `i18n-embed`

## v0.6.1

### Bug Fixes

+ Only re-export optional dependencies when they're actually enabled in the crate features ([#26](https://github.com/kellpossible/cargo-i18n/pull/26) thanks to @jplatte.)

## v0.6.0

+ Changed the argument for `LanguageRequester::add_listener()` to use a `std::rc::Weak` instead of `std::rc::Rc` to make it more obvious that it is the caller's responsibility to hold on to the `Rc` in order to maintain the reference.
+ Fixed typo in `LanguageRequester::set_language_override()`.

## v0.5.0

+ Refactored `I18nEmbedError::Multiple(Box<Vec<I18nEmbedError>>)` to `I18nEmbedError::Multiple(Vec<I18nEmbedError>)`, removing the useless box (and complaining Clippy lint).
+ Refactored `select()` method to use slice argument instead of `&Vec<LanguageIdentifier>`.
+ Changed `LanguageRequester::add_listener(&mut self, localizer: &Rc<Box<dyn Localizer<'a>>>)` to `add_listener(&mut self, localizer: &Rc<dyn Localizer<'a>>)` removing the unnecessary `Box`.
+ Added `Default` implementation for `DesktopLanguageRequester`.

## v0.4.2

+ Update `fluent-langneg` dependency to version `0.13`.
+ Update `unic-langid` dependency to version `0.9`.
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
