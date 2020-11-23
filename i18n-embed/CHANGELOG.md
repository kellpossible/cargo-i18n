# Changelog for `i18n-embed`

## Master

### Internal Changes

+ Close #36, remove the now redundant CRLF fix.

## v0.9.3

### Fixes

+ Updated documentation for `select()` function.

## v0.9.2

### Fixes

+ Remove compiler warning.

## v0.9.1

### Fixes

+ Renamed argument in `select()` method for clarity.
+ Changed logs in `select()` method to use `debug` level instead of `info` level.

## v0.9.0

+ Bumped version to reflect potential breaking changes present in the new version of `fluent`, `0.13` which is exposed in this crate's public API. And yanked previous versions of `i18n-embed`: `0.8.6` and `0.8.5`.

## v0.8.6

+ Update documentation and example to more accurately reflect the current state of `LangaugeRequester::poll()` on various systems.

## v0.8.5

### New Features

+ Add new `get_args_fluent()` method to `FluentLanguageLoader` to allow arguments to be specified using `fluent`'s new `FluentArgs` type.

### Internal Changes

+ Update `fluent` to version `0.13`.
+ Fixes to address breaking changes in `fluent-syntax` version `0.10`.

## v0.8.4

### Bug Fixes

+ A workaround for the [fluent issue #191](https://github.com/projectfluent/fluent-rs/issues/191), where CRLF formatted localization files are not always successfully parsed by fluent.

## v0.8.3

### New Features

+ Added a new `with_mesage_iter()` method to `FluentLanguageLoader`, to allow iterating over the messages available for a particular language.
+ Added `Default` implementation for `WebLanguageRequester`.

## v0.8.2

+ Fixed some mistakes in the docs.

## v0.8.1

+ Update version reference to `i18n-embed` in README, and docs.

## v0.8.0

Changes to support the new `i18n-embed-fl` crate's `fl!()` macro, and some major cleanup/refactoring/simplification.

### New Features

+ A new `I18nAssets` trait, to support situations where assets are not embedded.
+ Automatic implementation of the `I18nAssets` trait for types that implement `RustEmbed`.
+ A new `FileSystemAssets` type (which is enabled using the crate feature `filesystem-assets`), which implements `I18nAssets` for loading assets at runtime from the file system.
+ Implemented `Debug` trait on more types.
+ Added new `has()` and `with_fluent_message()` methods to `FluentLanguageLoader`.
+ Made `LanguageRequesterImpl` available on default crate features. No longer requires `gettext-system` or `fluent-system` to be enabled.

### Breaking Changes

+ Removed `I18nEmbed` trait, and derive macro, it was replaced with the new `I18nAssets` trait.
+ Clarified the `domain` and `module` arguments/variable inputs to `FluentLanguageLoader` and `GettextLanguageLoader`, and in the `LanguageLoader` trait with some renaming.
+ Removed a bunch of unecessary lifetimes, and `'static` bounds on types, methods and arguments.
+ `LanguageRequester::current_languages()`'s return type now uses `String` as the `HashMap` key instead of `&'static str`.
+ `available_languages()` implementation moved from `I18nEmbed` to `LanguageLoader`.

### Bug Fixes

+ Improved resolution of `i18n.toml` location in both the `gettext_language_loader!()` and `fluent_language_loader!()` macros using [find-crate](https://github.com/taiki-e/find-crate).

## v0.7.2

+ Fix broken documentation links when compiling with no features.

## v0.7.1

+ Fix broken documentation links.

## v0.7.0

Changes for the support of the `fluent` localization system.

### New Features

+ Added two new optional crate feature flags `gettext-system` and `fluent-system` to enable the new `GettextLanguageLoader` and `FluentLanguageLoader` types. See the [README](./README.md) and docs for more details.

### Breaking Changes

+ Update to `i18n-config` version `0.3.0`, contains breaking changes to `i18n.toml` configuration file format. See the [i18n changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-config/CHANGELOG.md#v030) for more details.
+ Rename `language_loader!()` macro to `gettext_language_loader!()`, and change how it works a little to make it simpler. Most of the functionality has been moved into the new `GettextLanguageLoader` type. See the docs.
+ `gettext-system` is no longer included in the default crate features.

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

Mostly a refactor of `LanguageLoader` and `I18nAssets` to solve [issue #15](https://github.com/kellpossible/cargo-i18n/issues/15).

+ Replaced the derive macro for `LanguageLoader` with a new `language_loader!(StructName)` which creates a new struct with the specified `StructName` and implements `LanguageLoader` for it. This was done because `LanguageLoader` now needs to store state for the currently selected language, and deriving this automatically would be complicated.
+ Refactored `I18nAssets` to move the `load_language_file` responsibility into `LanguageLoader` and add a new `load_language` method to `LanguageLoader`.
+ Refactored `I18nAssetsDyn` to also expose the `RustEmbed#get()` method, required for the new `LanguageLoader` changes.
+ Using `LanguageLoader` as a static now requires [lazy_static](https://crates.io/crates/lazy_static) or something similar because the `StructName#new()` constructor which is created for it in `language_loader!(StructName)` is not `const`.

## v0.3.4

+ Made `WebLanguageRequester::requested_languages()` public.

## v0.3.3

+ Updated link to this changelog in the crate README.

## v0.3.2

+ Bump `i18n-config` dependency in `i18n-embed-impl` version to `0.2`.
