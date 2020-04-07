# cargo-i18n [![crates.io badge](https://img.shields.io/crates/v/cargo-i18n.svg)](https://crates.io/crates/cargo-i18n) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/LICENSE.txt) [![docs.rs badge](https://docs.rs/cargo-i18n/badge.svg)](https://docs.rs/cargo-i18n/)

This crate is a Cargo sub-command `cargo i18n` which can be used to extract and build localization resources for your crate.

The `cargo i18n` command reads the configuration file (by default called `i18n.toml`) in the root directory of your crate, and then proceeds to extract  localization resources from your source files, and build them.

The [i18n-embed](https://crates.io/crates/i18n-embed) library has been created to allow you to conveniently embed the localizations in your application or library, and have them selected at runtime.

The [i18n-build](https://crates.io/crates/i18n-build) library contains most of the implementation for this tool. It has been published separately to allow its direct use within project build scripts if required.

## i18n.toml Configuration

Available configuration options for `i18n.toml`:

```toml
# (Required) The locale/language identifier of the language used in the source
# code.
src_locale = "en-US"

# (Required) The locales that the software will be translated into.
target_locales = ["es", "ru", "cz"]

# (Optional) Specify which subcrates to perform localization within. If the
# subcrate has its own `i18n.toml` then, it will have its localization
# performed independently (rather than being incorporated into the parent
# project's localization).
subcrates = ["subcrate1", "subcrate2"]

# (Optional) Use the gettext localization system.
[gettext]
# (Required) Path to the output directory, relative to `i18n.toml` of the crate
# being localized.
output_dir = "i18n"

# (Optional) The reporting address for msgid bugs. This is the email address or
# URL to which the translators shall report bugs in the untranslated
# strings.
msg_bugs_address = "example@example.com"

# (Optional) Set the copyright holder for the generated files.
copyright_holder = "You?"

# (Optional) If this crate is being localized as a subcrate, store the final
# localization artifacts (the module pot and mo files) with the parent crate's
# output. Currently crates which contain subcrates with duplicate names are not
# supported.
extract_to_parent = true

# (Optional) If a subcrate has extract_to_parent set to true, then merge the
# output pot file of that subcrate into this crate's pot file.
collate_extracted_subcrates = true

# (Optional) Whether or not to perform string extraction using the `xtr` tool.
xtr = true

# (Optional )Path to where the pot files will be written to by `xtr` command,
# and were they will be read from by the `msginit` and `msgmerge` commands. By
# default this is `output_dir/pot`.
pot_dir = "i18n/pot"

# (Optional) Path to where the po files will be stored/edited with the
# `msgmerge` and `msginit` commands, and where they will be read from with the
# `msgfmt` command. By default this is `output_dir/po`.
po_dir = "i18n/po"

# (Optional) Path to where the mo files will be written to by the `msgfmt`
# command. By default this is `output_dir/mo`.
mo_dir = "i18n/mo"
```

## Contributing

Pull-requests are welcome, and you can also contribute to the localization of this tool via:

+ [POEditor - cargo-i18n](https://poeditor.com/join/project/J7NiRCGpXa)
+ [POEditor - i18n-build](https://poeditor.com/join/project/BCW39cVoco)

Or you can also use your favourite `po` editor directly to help with localizing the files located in [i18n/po](./i18n/po) and [i18n-build/i18n/po](./i18n-build/i18n/po).
