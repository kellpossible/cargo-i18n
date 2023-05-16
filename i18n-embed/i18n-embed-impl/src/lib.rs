/// A procedural macro to create a new `GettextLanguageLoader` using
/// the current crate's `i18n.toml` configuration, and domain.
///
/// ⚠️ *This API requires the following crate features to be
/// activated: `gettext-system`.*
///
/// ## Example
///
/// ```ignore
/// use i18n_embed::gettext::{gettext_language_loader, GettextLanguageLoader};
/// let my_language_loader: GettextLanguageLoader = gettext_language_loader!();
/// ```
#[proc_macro]
#[cfg(feature = "gettext-system")]
pub fn gettext_language_loader(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
    let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");

    // Special case for when this macro is invoked in i18n-embed tests/docs
    let i18n_embed_crate_name = if current_crate_package.name == "i18n_embed" {
        "i18n_embed".to_string()
    } else {
        manifest
            .find(|s| s == "i18n-embed")
            .expect("i18n-embed should be an active dependency in your Cargo.toml")
            .name
    };

    let i18n_embed_crate_ident =
        syn::Ident::new(&i18n_embed_crate_name, proc_macro2::Span::call_site());

    let config_file_path = i18n_config::locate_crate_paths()
        .unwrap_or_else(|error| {
            panic!(
                "gettext_language_loader!() is unable to locate i18n config file: {}",
                error
            )
        })
        .i18n_config_file;

    let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
        panic!(
            "gettext_language_loader!() had a problem reading i18n config file {0:?}: {1}",
            std::fs::canonicalize(&config_file_path).unwrap_or_else(|_| config_file_path.clone()),
            err
        )
    });

    if config.gettext.is_none() {
        panic!(
            "gettext_language_loader!() had a problem parsing i18n config file {0:?}: there is no `[gettext]` section",
            std::fs::canonicalize(&config_file_path).unwrap_or(config_file_path)
        )
    }

    let fallback_language = syn::LitStr::new(
        &config.fallback_language.to_string(),
        proc_macro2::Span::call_site(),
    );

    let gen = quote::quote! {
        #i18n_embed_crate_ident::gettext::GettextLanguageLoader::new(
            module_path!(),
            #fallback_language.parse().unwrap(),
        )
    };

    gen.into()
}

/// A procedural macro to create a new `FluentLanguageLoader` using
/// the current crate's `i18n.toml` configuration, and domain.
///
/// ⚠️ *This API requires the following crate features to be
/// activated: `fluent-system`.*
///
/// ## Example
///
/// ```ignore
/// use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
/// let my_language_loader: FluentLanguageLoader = fluent_language_loader!();
/// ```
#[proc_macro]
#[cfg(feature = "fluent-system")]
pub fn fluent_language_loader(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
    let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");

    // Special case for when this macro is invoked in i18n-embed tests/docs
    let i18n_embed_crate_name = if current_crate_package.name == "i18n_embed" {
        "i18n_embed".to_string()
    } else {
        manifest
            .find(|s| s == "i18n-embed")
            .expect("i18n-embed should be an active dependency in your Cargo.toml")
            .name
    };

    let i18n_embed_crate_ident =
        syn::Ident::new(&i18n_embed_crate_name, proc_macro2::Span::call_site());

    let config_file_path = i18n_config::locate_crate_paths()
        .unwrap_or_else(|error| {
            panic!(
                "fluent_language_loader!() is unable to locate i18n config file: {}",
                error
            )
        })
        .i18n_config_file;

    let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
        panic!(
            "fluent_language_loader!() had a problem reading i18n config file {0:?}: {1}",
            std::fs::canonicalize(&config_file_path).unwrap_or_else(|_| config_file_path.clone()),
            err
        )
    });

    if config.fluent.is_none() {
        panic!(
            "fluent_language_loader!() had a problem parsing i18n config file {0:?}: there is no `[fluent]` section",
            std::fs::canonicalize(&config_file_path).unwrap_or(config_file_path)
        )
    }

    let fallback_language = syn::LitStr::new(
        &config.fallback_language.to_string(),
        proc_macro2::Span::call_site(),
    );

    let domain_str = config
        .fluent
        .and_then(|f| f.domain)
        .unwrap_or(current_crate_package.name);
    let domain = syn::LitStr::new(&domain_str, proc_macro2::Span::call_site());

    let gen = quote::quote! {
        #i18n_embed_crate_ident::fluent::FluentLanguageLoader::new(
            #domain,
            #fallback_language.parse().unwrap(),
        )
    };

    gen.into()
}
