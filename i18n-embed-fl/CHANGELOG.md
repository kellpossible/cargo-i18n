# Changelog for `i18n-embed-fl`

## master

### Breaking Changes

+ Updated `fluent` to version `0.15`.

## v0.4.0

### Breaking Changes

+ Update `i18n-embed` to version `0.11`.

### Internal Changes

+ Refactoring during the fix for [#60](https://github.com/kellpossible/cargo-i18n/issues/60).

## v0.3.1

### Internal Changes

+ Safer use of DashMap's new `4.0` API thanks to [#56](https://github.com/kellpossible/cargo-i18n/pull/56).

## v0.3.0

+ Update `fluent` dependency to version `0.14`.
+ Update to `dashmap` version `4.0`, and fix breaking change.

## v0.2.0

+ Bumped version to reflect potential breaking changes present in the new version of `fluent`, `0.13` which is exposed in this crate's public API. And yanked previous version of `i18n-embed-fl`: `0.1.6`.

## v0.1.6

### Internal Changes

+ Update to `fluent` version `0.13`.
+ Fixes to address breaking changes in `fluent-syntax` version `0.10`.

## v0.1.5

### New Features

+ Updated readme with example convenience wrapper macro.
+ Added suggestions for message ids (ranked by levenshtein distance) to the error message when the current one fails to match.

## v0.1.4

+ Enable the args hashmap option `fl!(loader, "message_id", args())` to be parsed as an expression, instead of just an ident.

## v0.1.3

+ Fix bug where message check wasn't occurring with no arguments or with hashmap arguments.

## v0.1.2

+ Change the `loader` argument to be an expression, instead of an ident, so it allows more use cases.

## v0.1.1

+ Remove `proc_macro_diagnostic` feature causing problems compiling on stable, and use `proc-macro-error` crate instead.

## v0.1.0

+ Initial version, introduces the `fl!()` macro.
