# cargo-i18n

This crate is a Cargo subcommand `cargo i18n` which can be used to extract and build localization resources for your crate.

The `cargo i18n` command reads the `i18n.toml` config in your crate root, and based on the configuration there, proceeds to extract localization resources, and build them.

An example `i18n.toml` config:

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

Pull-requests are welcome, and you can also contribute to the localization of this library via:

+ [POEditor - cargo-i18n](https://poeditor.com/join/project/J7NiRCGpXa)
+ [POEditor - i18n-build](https://poeditor.com/join/project/BCW39cVoco)

Or you can also use your favourite `po` editor directly to help with localizing the files located in [i18n/po](./i18n/po) and [i18n-build/i18n/po](./i18n-build/i18n/po).