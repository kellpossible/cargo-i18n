# cargo-i18n [![crates.io badge](https://img.shields.io/crates/v/cargo-i18n.svg)](https://crates.io/crates/cargo-i18n) [![license badge](https://img.shields.io/github/license/kellpossible/cargo-i18n)](https://github.com/kellpossible/cargo-i18n/blob/master/LICENSE.txt) [![github actions badge](https://github.com/kellpossible/cargo-i18n/workflows/Rust/badge.svg)](https://github.com/kellpossible/cargo-i18n/actions?query=workflow%3ARust)

This crate is a Cargo sub-command `cargo i18n` which can be used to extract and
build, and verify localization resources at compile time for your crate. The
[i18n-embed](https://crates.io/crates/i18n-embed) library has been created to
allow you to conveniently embed these localizations into your application or
library, and have them selected at runtime. Different systems can be used simultaneously.

`i18n-embed` supports both the following localization systems:

+ [fluent](https://www.projectfluent.org/)
+ [gettext](https://www.gnu.org/software/gettext/)

You can install this tool using the command: `cargo install cargo-i18n`.

The `cargo i18n` command reads the configuration file (by default called `i18n.toml`) in the root directory of your crate, and then proceeds to extract  localization resources from your source files, and build them.

The [i18n-build](https://crates.io/crates/i18n-build) library contains most of the implementation for this tool. It has been published separately to allow its direct use within project build scripts if required.

**[Changelog](https://github.com/kellpossible/cargo-i18n/releases)**

## Usage with Fluent

Fluent support is now available in [i18n-embed](./i18n-embed/README.md). See the examples for that crate for how to use it.

Currently there are no validations performed by `cargo-i18n` when using the `fluent` localization system (see tracking issue [#31](https://github.com/kellpossible/cargo-i18n/issues/31)).

## Usage with Gettext

This is an example for how to use the `cargo-i18n` tool, and `i18n-embed` the `gettext` localization tool system. Please note that the `gettext` localization system is technically inferior to `fluent` [in a number of ways](https://github.com/projectfluent/fluent/wiki/Fluent-vs-gettext), however there are always legacy reasons, and the developer/translator ecosystem around `gettext` is mature.

Firstly, ensure you have the required utilities installed on your system. See [Gettext System Requirements](#Gettext-Requirements) and install the necessary utilities and commands for the localization system you will be using.

### Defining Localized Strings

You will need to ensure that your strings in your source code that you want localized are using the `tr!()` macro from the [tr](https://crates.io/crates/tr) crate.

You can add comments to your strings which will be available to translators to add context, and ensure that they understand what the string is for.

For example:

```rust
use tr::tr;

fn example(file: String) {
    let my_string = tr!(
        // {0} is a file path
        // Example message: Printing this file: "file.doc"
        "Printing this file: \"{0}\"",
        file
    );
}
```

### Minimal Configuration

You will need to create an `i18n.toml` configuration in the root directory of your crate. A minimal configuration for a binary crate to be localized to Spanish and Japanese using the `gettext` system would be:

```toml
# (Required) The language identifier of the language used in the
# source code for gettext system, and the primary fallback language
# (for which all strings must be present) when using the fluent
# system.
fallback_language = "en"

[gettext]
# (Required) The languages that the software will be translated into.
target_languages = ["es", "ja"]

# (Required) Path to the output directory, relative to `i18n.toml` of the crate
# being localized.
output_dir = "i18n"
```

See [Configuration](#Configuration) for a description of all the available configuration options.

### Running `cargo i18n`

Open your command line/terminal and navigate to your crate directory, and run `cargo i18n`. You may be prompted to enter some email addresses to use for contact points for each of the language's `po` files. At the end there should be a new directory in your crate called `i18n`, and inside will be `pot`, `po` and `mo` directories.

The `pot` directory contains `pot` files which were extracted from your source code using the `xtr` tool, and there should be a single `pot` file with the name of your crate in here too, which is the result of merging all the other `pot` files.

The `po` directory contains the language specific message files.

The `mo` directory contains the compiled messages, which will later be embedded into your application.

At this point it could be a good idea to add the following to your crate's `.gitignore` (if you are using git):

```gitignore
/i18n/pot
/i18n/mo
```

If you want your crate to be able to build without requiring this tool to be present on the system, then you can leave the `/i18n/mo` directory out of the `.gitignore`, and commit the files inside.

### Embedding Translations

Now that you have compiled your translations, you can embed them within your application. For this purpose the [i18n-embed](https://crates.io/crates/i18n-embed) crate was created.

Add the following to your `Cargo.toml` dependencies:

```toml
[dependencies]
i18n-embed = "0.7"
```

A minimal example for how to embed the compiled translations into your application could be:

```rust
use i18n_embed::{I18nEmbed, DesktopLanguageRequester,
    gettext::gettext_language_loader};
use rust_embed::RustEmbed;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"] // path to the compiled localization resources
struct Translations;

fn main() {
    let translations = Translations {};
    let language_loader = gettext_language_loader!();

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let requested_languages = DesktopLanguageRequester::requested_languages();

    i18n_embed::select(&language_loader, &translations, &requested_languages);

    // continue with your application
}
```

You can see the [i18n-embed documentation](https://docs.rs/i18n-embed/) for more detailed examples of how this library can be used.

### Distributing to Translators

Now you need to send of the `po` files to your translators, or provide them access to edit them. Some desktop tools which can be used for the translation include:

+ [poedit](https://poedit.net/)
+ [Qt Linguist](https://doc.qt.io/qt-5/linguist-translators.html) ([Windows build](https://github.com/thurask/Qt-Linguist))

Or you could also consider setting up a translation management website for your project to allow translators to edit translations without requiring them to interact with source control or mess around with sending files and installing applications. Some examples:

**Self Hosted:**

+ [pootle](https://pootle.translatehouse.org/)
+ [weblate](https://weblate.org/) - also a cloud offering.

**Cloud:**

+ [poeditor](https://poeditor.com/projects/) - free for open source projects, currently being used for this project.
+ [crowdin](https://crowdin.com/) - free for popular open source projects.

### Updating Translations

Once you have some updated `po` files back from translators, or you want to update the `po` files with new or edited strings, all you need to do is run `cargo i18n` to update the `po` files, and recompile updated `mo` files, then rebuild your application with `cargo build`.

For some projects using build scripts, with complex pipelines, and with continuous integration, you may want to look into using the [i18n-build](https://crates.io/crates/i18n-build) for automation as an alternative to the `cargo i18n` command line tool.

## Example Projects

For a complete example usage, including localizing sub-crates as libraries, you can see the [source code](https://github.com/kellpossible/cargo-i18n/) for this project, which localizes itself. This project was originally created to aid in the localization for the [coster (work in progress)](https://github.com/kellpossible/coster) self-hosted web application.

## Configuration

Available configuration options for `i18n.toml`:

```toml
# (Required) The language identifier of the language used in the
# source code for gettext system, and the primary fallback language
# (for which all strings must be present) when using the fluent
# system.
fallback_language = "en-US"

# (Optional) Specify which subcrates to perform localization within. If the
# subcrate has its own `i18n.toml` then, it will have its localization
# performed independently (rather than being incorporated into the parent
# project's localization).
subcrates = ["subcrate1", "subcrate2"]

# (Optional) Use the gettext localization system.
[gettext]
# (Required) The languages that the software will be translated into.
target_languages = ["es", "ru", "cz"]

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
# supported. By default this is false.
extract_to_parent = false

# (Optional) If a subcrate has extract_to_parent set to true, then merge the
# output pot file of that subcrate into this crate's pot file. By default this
# is false.
collate_extracted_subcrates = false

# (Optional) How much message location information to include in the output.
# If the type is ‘full’ (the default), it generates the lines with both file
# name and line number: ‘#: filename:line’. If it is ‘file’, the line number
# part is omitted: ‘#: filename’. If it is ‘never’, nothing is generated.
# [possible values: full, file, never].
add_location = "full"

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

## System Requirements

### Gettext Requirements

Using the `gettext` localization system with this tool requires you to have [gettext](https://www.gnu.org/software/gettext/) installed on your system.

The [`msginit`](https://www.gnu.org/software/gettext/manual/html_node/msginit-Invocation.html), [`msgfmt`](https://www.gnu.org/software/gettext/manual/html_node/msgfmt-Invocation.html), [`msgmerge`](https://www.gnu.org/software/gettext/manual/html_node/msgmerge-Invocation.html) and [`msgcat`](https://www.gnu.org/software/gettext/manual/html_node/msgcat-Invocation.html) commands all need to be installed and present in your path.

You also need to ensure that you have the [xtr](https://crates.io/crates/xtr)
string extraction command installed, which can be achieved using `cargo install
xtr`.

## Contributing

Pull-requests are welcome, but for design changes it is preferred that you create a [GitHub issue](https://github.com/kellpossible/cargo-i18n/issues) first to discuss it before implementation. You can also contribute to the localization of this tool via:

+ [POEditor - cargo-i18n](https://poeditor.com/join/project/J7NiRCGpXa)
+ [POEditor - i18n-build](https://poeditor.com/join/project/BCW39cVoco)

Or you can also use your favourite `po` editor directly to help with localizing the files located in [i18n/po](./i18n/po) and [i18n-build/i18n/po](./i18n-build/i18n/po).

To add a new language, you can make a request via a GitHub issue, or submit a pull request adding the new locale to [i18n.toml](https://github.com/kellpossible/cargo-i18n/blob/master/i18n.toml) and generating the associated new `po` files using `cargo i18n`.

Translations of this [README.md](./README.md) file are also welcome, and can be submitted via pull request. Just name it `README.md.lang`, where `lang` is the locale code (see [List of ISO 639-1 codes](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes)).

## Authors

+ [Contributors](https://github.com/kellpossible/cargo-i18n/graphs/contributors)
+ [Translators](https://github.com/kellpossible/cargo-i18n/blob/master/i18n/TRANSLATORS)
