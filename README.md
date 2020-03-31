# cargo-i18n

This crate is a Cargo subcommand `cargo i18n` which can be used to extract and build localization resources for your crate.

The `cargo i18n` command reads the `i18n.toml` config in your crate root, and based on the configuration there, proceeds to extract localization resources, and build them.

An example `i18n.toml` config:

```toml
# (Required) The locale/language identifier of the language used in the source code.
src_locale = "en-US"

# (Required) The locales that the software will be translated into.
target_locales = ["es", "ru", "cz"]

# (Optional) Specify which subcrates to perform localization within. If the subcrate has its own `i18n.toml` then, it will have its localization performed  independently (rather than being incorporated into the parent project's localization).
subcrates = ["subcrate1", "subcrate2"]

# (Optional) Use the gettext localization system.
[gettext]
# (Required) Path to the output directory, relative to `i18n.toml` of the crate being localized.
output_dir = "i18n"

# (Optional) Whether or not to perform string extraction using the `xtr` tool.
xtr = true

# (Optional) Path to where the po files will be stored/edited with the `msgmerge` and `msginit` commands, and where they will be read from with the `msgfmt` command. By default this `output_dir/po`.
po_dir = "i18n/po"
```


