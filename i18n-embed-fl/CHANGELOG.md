# Changelog for `i18n-embed-fl`

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
