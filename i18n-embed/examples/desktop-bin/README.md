# `bin` `i18n-embed` Example

This example demonstrates how to use a library localized with [i18n-embed](../../i18n-embed/) and the `DesktopLanguageRequester` in a desktop CLI application.

It can use either of two backends: a library with an `i18n.toml` file in the crate root, or an identical library with the `i18n.toml` in a subdirectory of the crate root. The first is used by default; to run the second, use `cargo run --no-default-features --features nonstandard_config`.

On unix, you can override the detected language by setting the `LANG` environment variable before running. The two available languages in the `library-fluent` example are `fr`, `eo`, and `en` (the fallback).
