# Changelog for `i18n-embed`

## master

### Fixes

+ Fallback to `std::env::var("CARGO_PKG_NAME")` Fixes [#97](https://github.com/kellpossible/cargo-i18n/issues/97)

## v0.14.1

## Internal

+ Relax the version constraint on `arc-swap`.

## v0.14.0

### Internal

+ Bump dependencies and use workspace dependencies.

## v0.13.9

## New Features

- Add option to override the default domain name for fluent assets.

## v0.13.8

### Internal

- Made `arc-swap` an optional dependency, it's only required for `fluent-system` implementation.

## v0.13.7

### New Features

- A new `LanguageLoader::load_available_languages()` method to load all available languages.
- A new `FluentLanguageLoader::select_languages()` method (renamed `FluentLanguageLoader::lang()`).
- A new `FluentLanguageLoader::select_languages_negotiate()` method to select languages based on a negotiation strategy using the available languges.

### Deprecated

- `FluentLanguageLoader::lang()` is deprecated in favour of renamed `FluentLanguageLoader::select_languages()`.

## v0.13.6

### New Features

- A single new method called `FluentLanguageLoader::lang()` thanks to [@bikeshedder](https://github.com/bikeshedder)!

This methods allows creating a shallow copy of the
FluentLanguageLoader which can than be used just like the original
loader but with a different current language setting. That makes it
possible to use the fl! macro without any changes and is a far more
elegant implementation than adding multiple get_lang\* methods as
done in #84.

### Deprecated

- `FluentLanguageLoader::get_lang*` methods have been deprecated in favour of `FluentLanguageLoader::lang()`.

## v0.13.5

### New Features

- Support fluent attributes [#98](https://github.com/kellpossible/cargo-i18n/pull/98) thanks to [@Almost-Senseless-Coder](https://github.com/Almost-Senseless-Coder)!
  - New methods on `FluentLanguageLoader`:
  - `get_attr`
  - `get_attr_args_concrete`
  - `get_attr_args_fluent`
  - `get_attr_args`
  - `get_lang_attr`
  - `get_lang_attr_args_concrete`
  - `get_lang_attr_args_fluent`
  - `get_lang_attr_args`

### Internal

- Bump `env_logger` dev dependency to version `0.10`.
- Fix clippy warnings.

## v0.13.4

### New Features

- Implement `FluentLanguageLoader::get_lang(...)` methods. This enables the use of the fluent language loader without using the global current language setting, which is useful for web servers. Closes [#59](https://github.com/kellpossible/cargo-i18n/issues/59).

## v0.13.3

- Update `rust-embed` to `6.3` to address [RUSTSEC-2021-0126](https://rustsec.org/advisories/RUSTSEC-2021-0126.html).

## v0.13.2

### Internal

- Use conditional compilation correctly for doctests.
- Update `parking_lot` to version `0.12`.

## v0.13.1

### New Features

- New `FluentLanguageLoader::with_bundles_mut()` method to allow mutable access to bundles.

### Internal Changes

- Bumped `pretty_assertions` version `1.0`.
- Fixed clippy lints.

## v0.13.0

### Breaking Changes

- Update `rust-embed` to version `6`.
- Update `fluent` to version `0.16`.

## v0.12.1

### Documentation

- Updated crate description.
- Don't reference specific `i18n-embed` version number.

## v0.12.0

### Documentation

- Added [`bin`](./examples/bin/) example which explains how to consume the [`lib-fluent`](./examples/lib-fluent) example library in a desktop CLI application.

### Breaking Changes

- Updated `fluent` to version `0.15`.

### Internal Changes

- Updated `FluentLanguageLoader` to use a thread safe [IntlLangMemoizer](https://docs.rs/intl-memoizer/0.5.1/intl_memoizer/concurrent/struct.IntlLangMemoizer.html) as per the notes on [FluentBundle's concurrency](https://docs.rs/fluent-bundle/0.15.0/fluent_bundle/bundle/struct.FluentBundle.html#concurrency). This was required to solve a compilation error in `i18n-embed-fl` and may also fix problems for other downstream users who were expecting `FluentLangaugeLoader` to be `Send + Sync`. It might impact performance for those who are not using this in multi-threaded context, please report this, and in which case support for switching the `IntlLangMemoizer` added.

## v0.11.0

### Documentation

- Updated/improved examples, including an improvement for how to expose localization from a library using the `fluent` system, ensuring that the fallback language is loaded by default.
- Updated examples with new `LanguageRequester::add_listener()` signature now using `std::sync::Arc` and `std::sync::Weak`.

### Breaking Changes

- Fix for [#60](https://github.com/kellpossible/cargo-i18n/issues/60) where `i18n-embed-fl` loads `i18n.toml` from a different path to `fluent_language_loader!()`. For subcrates in a workspace previously `fluent_language_loader!()` and `gettext_language_loader!()` were searching for `i18n.toml` in the crate root of the workspace, rather than the current subcrate. This was not the expected behaviour. This fix could be considered a breaking change for libraries that inadvertently relied on that behaviour.
- For `LanguageRequester::add_listener()` change the `listener` type to `std::sync::Weak`, to better support temporary dependencies that may come from another thread. This should not be a performance bottleneck. This also affects `DesktopLanguageRequester` and `WebLanguageRequester`.

### New Features

- New `LanguageRequester::add_listener_ref()` method to add permenant listeners of type `&dyn Localizer`. This also affects `DesktopLanguageRequester` and `WebLanguageRequester`.

### Internal Changes

- Fix clippy warnings.
- Update `i18-embed-impl` to version `0.7.0`.

## v0.10.2

- Add workaround for [#57](https://github.com/kellpossible/cargo-i18n/issues/57) for until <https://github.com/projectfluent/fluent-rs/issues/213> is solved.

## v0.10.1

- Update references to `i18n-embed` version in readme and source code examples.

## v0.10.0

### Fixes

- More gracefully handle the situation on Linux where LANG environment variable is not set due to [rust-locale/locale_config#6](https://github.com/rust-locale/locale_config/issues/6). Fixes [#49](https://github.com/kellpossible/cargo-i18n/issues/49).

### Internal Changes

- Update `fluent` dependency to version `0.14`.

## v0.9.4

### New Features

- Functionality to disable bidirectional isolation in Fluent with `FluentLanguageLoader` with a new `set_use_isolating` method [#45](https://github.com/kellpossible/cargo-i18n/issues/45).

### Internal Changes

- Remove the now redundant CRLF fix [#36](https://github.com/kellpossible/cargo-i18n/issues/36).

## v0.9.3

### Fixes

- Updated documentation for `select()` function.

## v0.9.2

### Fixes

- Remove compiler warning.

## v0.9.1

### Fixes

- Renamed argument in `select()` method for clarity.
- Changed logs in `select()` method to use `debug` level instead of `info` level.

## v0.9.0

- Bumped version to reflect potential breaking changes present in the new version of `fluent`, `0.13` which is exposed in this crate's public API. And yanked previous versions of `i18n-embed`: `0.8.6` and `0.8.5`.

## v0.8.6

- Update documentation and example to more accurately reflect the current state of `LangaugeRequester::poll()` on various systems.

## v0.8.5

### New Features

- Add new `get_args_fluent()` method to `FluentLanguageLoader` to allow arguments to be specified using `fluent`'s new `FluentArgs` type.

### Internal Changes

- Update `fluent` to version `0.13`.
- Fixes to address breaking changes in `fluent-syntax` version `0.10`.

## v0.8.4

### Bug Fixes

- A workaround for the [fluent issue #191](https://github.com/projectfluent/fluent-rs/issues/191), where CRLF formatted localization files are not always successfully parsed by fluent.

## v0.8.3

### New Features

- Added a new `with_mesage_iter()` method to `FluentLanguageLoader`, to allow iterating over the messages available for a particular language.
- Added `Default` implementation for `WebLanguageRequester`.

## v0.8.2

- Fixed some mistakes in the docs.

## v0.8.1

- Update version reference to `i18n-embed` in README, and docs.

## v0.8.0

Changes to support the new `i18n-embed-fl` crate's `fl!()` macro, and some major cleanup/refactoring/simplification.

### New Features

- A new `I18nAssets` trait, to support situations where assets are not embedded.
- Automatic implementation of the `I18nAssets` trait for types that implement `RustEmbed`.
- A new `FileSystemAssets` type (which is enabled using the crate feature `filesystem-assets`), which implements `I18nAssets` for loading assets at runtime from the file system.
- Implemented `Debug` trait on more types.
- Added new `has()` and `with_fluent_message()` methods to `FluentLanguageLoader`.
- Made `LanguageRequesterImpl` available on default crate features. No longer requires `gettext-system` or `fluent-system` to be enabled.

### Breaking Changes

- Removed `I18nEmbed` trait, and derive macro, it was replaced with the new `I18nAssets` trait.
- Clarified the `domain` and `module` arguments/variable inputs to `FluentLanguageLoader` and `GettextLanguageLoader`, and in the `LanguageLoader` trait with some renaming.
- Removed a bunch of unecessary lifetimes, and `'static` bounds on types, methods and arguments.
- `LanguageRequester::current_languages()`'s return type now uses `String` as the `HashMap` key instead of `&'static str`.
- `available_languages()` implementation moved from `I18nEmbed` to `LanguageLoader`.

### Bug Fixes

- Improved resolution of `i18n.toml` location in both the `gettext_language_loader!()` and `fluent_language_loader!()` macros using [find-crate](https://github.com/taiki-e/find-crate).

## v0.7.2

- Fix broken documentation links when compiling with no features.

## v0.7.1

- Fix broken documentation links.

## v0.7.0

Changes for the support of the `fluent` localization system.

### New Features

- Added two new optional crate feature flags `gettext-system` and `fluent-system` to enable the new `GettextLanguageLoader` and `FluentLanguageLoader` types. See the [README](./README.md) and docs for more details.

### Breaking Changes

- Update to `i18n-config` version `0.3.0`, contains breaking changes to `i18n.toml` configuration file format. See the [i18n changelog](https://github.com/kellpossible/cargo-i18n/blob/master/i18n-config/CHANGELOG.md#v030) for more details.
- Rename `language_loader!()` macro to `gettext_language_loader!()`, and change how it works a little to make it simpler. Most of the functionality has been moved into the new `GettextLanguageLoader` type. See the docs.
- `gettext-system` is no longer included in the default crate features.

## v0.6.1

### Bug Fixes

- Only re-export optional dependencies when they're actually enabled in the crate features ([#26](https://github.com/kellpossible/cargo-i18n/pull/26) thanks to @jplatte.)

## v0.6.0

- Changed the argument for `LanguageRequester::add_listener()` to use a `std::rc::Weak` instead of `std::rc::Rc` to make it more obvious that it is the caller's responsibility to hold on to the `Rc` in order to maintain the reference.
- Fixed typo in `LanguageRequester::set_language_override()`.

## v0.5.0

- Refactored `I18nEmbedError::Multiple(Box<Vec<I18nEmbedError>>)` to `I18nEmbedError::Multiple(Vec<I18nEmbedError>)`, removing the useless box (and complaining Clippy lint).
- Refactored `select()` method to use slice argument instead of `&Vec<LanguageIdentifier>`.
- Changed `LanguageRequester::add_listener(&mut self, localizer: &Rc<Box<dyn Localizer<'a>>>)` to `add_listener(&mut self, localizer: &Rc<dyn Localizer<'a>>)` removing the unnecessary `Box`.
- Added `Default` implementation for `DesktopLanguageRequester`.

## v0.4.2

- Update `fluent-langneg` dependency to version `0.13`.
- Update `unic-langid` dependency to version `0.9`.
- Fix incorrect comment in code example [#18](https://github.com/kellpossible/cargo-i18n/issues/18).

## v0.4.0

Mostly a refactor of `LanguageLoader` and `I18nAssets` to solve [issue #15](https://github.com/kellpossible/cargo-i18n/issues/15).

- Replaced the derive macro for `LanguageLoader` with a new `language_loader!(StructName)` which creates a new struct with the specified `StructName` and implements `LanguageLoader` for it. This was done because `LanguageLoader` now needs to store state for the currently selected language, and deriving this automatically would be complicated.
- Refactored `I18nAssets` to move the `load_language_file` responsibility into `LanguageLoader` and add a new `load_language` method to `LanguageLoader`.
- Refactored `I18nAssetsDyn` to also expose the `RustEmbed#get()` method, required for the new `LanguageLoader` changes.
- Using `LanguageLoader` as a static now requires [lazy_static](https://crates.io/crates/lazy_static) or something similar because the `StructName#new()` constructor which is created for it in `language_loader!(StructName)` is not `const`.

## v0.3.4

- Made `WebLanguageRequester::requested_languages()` public.

## v0.3.3

- Updated link to this changelog in the crate README.

## v0.3.2

- Bump `i18n-config` dependency in `i18n-embed-impl` version to `0.2`.
