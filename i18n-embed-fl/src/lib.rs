use fluent::concurrent::FluentBundle;
use fluent::{FluentAttribute, FluentMessage, FluentResource};
use fluent_syntax::ast::{CallArguments, Expression, InlineExpression, Pattern, PatternElement};
use i18n_embed::{fluent::FluentLanguageLoader, FileSystemAssets, LanguageLoader};
use proc_macro::TokenStream;
use proc_macro_error2::{abort, emit_error, proc_macro_error};
use quote::quote;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::OnceLock,
};

#[cfg(feature = "dashmap")]
use dashmap::mapref::one::Ref;
#[cfg(not(feature = "dashmap"))]
use std::sync::{Arc, RwLock};

use syn::{parse::Parse, parse_macro_input, spanned::Spanned};
use unic_langid::LanguageIdentifier;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");

#[derive(Debug)]
enum FlAttr {
    /// An attribute ID got provided.
    Attr(syn::Lit),
    /// No attribute ID got provided.
    None,
}

impl Parse for FlAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            let fork = input.fork();
            fork.parse::<syn::Token![,]>()?;
            if fork.parse::<syn::Lit>().is_ok()
                && (fork.parse::<syn::Token![,]>().is_ok() || fork.is_empty())
            {
                input.parse::<syn::Token![,]>()?;
                let literal = input.parse::<syn::Lit>()?;
                Ok(Self::Attr(literal))
            } else {
                Ok(Self::None)
            }
        } else {
            Ok(Self::None)
        }
    }
}

#[derive(Debug)]
enum FlArgs {
    /// `fl!(LOADER, "message", "optional-attribute", args)` where `args` is a
    /// `HashMap<&'a str, FluentValue<'a>>`.
    HashMap(syn::Expr),
    /// ```ignore
    /// fl!(LOADER, "message", "optional-attribute",
    ///     arg1 = "value",
    ///     arg2 = value2,
    ///     arg3 = calc_value());
    /// ```
    KeyValuePairs {
        specified_args: Vec<(syn::LitStr, Box<syn::Expr>)>,
    },
    /// `fl!(LOADER, "message", "optional-attribute")` no arguments after the message id and optional attribute id.
    None,
}

impl Parse for FlArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;

            let lookahead = input.fork();
            if lookahead.parse::<syn::ExprAssign>().is_err() {
                let hash_map = input.parse()?;
                return Ok(FlArgs::HashMap(hash_map));
            }

            let mut args: Vec<(syn::LitStr, Box<syn::Expr>)> = Vec::new();

            while let Ok(expr) = input.parse::<syn::ExprAssign>() {
                let argument_name_ident_opt = match &*expr.left {
                    syn::Expr::Path(path) => path.path.get_ident(),
                    _ => None,
                };

                let argument_name_ident = match argument_name_ident_opt {
                    Some(ident) => ident,
                    None => {
                        return Err(syn::Error::new(
                            expr.left.span(),
                            "fl!() unable to parse argument identifier",
                        ))
                    }
                }
                .clone();

                let argument_name_string = argument_name_ident.to_string();
                let argument_name_lit_str =
                    syn::LitStr::new(&argument_name_string, argument_name_ident.span());

                let argument_value = expr.right;

                if args
                    .iter()
                    .any(|(key, _value)| argument_name_lit_str == *key)
                {
                    // There's no Clone implementation by default.
                    let argument_name_lit_str =
                        syn::LitStr::new(&argument_name_string, argument_name_ident.span());
                    return Err(syn::Error::new(
                        argument_name_lit_str.span(),
                        format!(
                            "fl!() macro contains a duplicate argument `{}`",
                            argument_name_lit_str.value()
                        ),
                    ));
                }
                args.push((argument_name_lit_str, argument_value));

                // parse the next comma if there is one
                let _result = input.parse::<syn::Token![,]>();
            }

            if args.is_empty() {
                let span = match input.fork().parse::<syn::Expr>() {
                    Ok(expr) => expr.span(),
                    Err(_) => input.span(),
                };
                Err(syn::Error::new(span, "fl!() unable to parse args input"))
            } else {
                args.sort_by_key(|(s, _)| s.value());
                Ok(FlArgs::KeyValuePairs {
                    specified_args: args,
                })
            }
        } else {
            Ok(FlArgs::None)
        }
    }
}

/// Input for the [fl()] macro.
struct FlMacroInput {
    fluent_loader: syn::Expr,
    message_id: syn::Lit,
    attr: FlAttr,
    args: FlArgs,
}

impl Parse for FlMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fluent_loader = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let message_id = input.parse()?;
        let attr = input.parse()?;
        let args = input.parse()?;

        Ok(Self {
            fluent_loader,
            message_id,
            attr,
            args,
        })
    }
}

struct DomainSpecificData {
    loader: FluentLanguageLoader,
    _assets: FileSystemAssets,
}

#[derive(Default)]
struct DomainsMap {
    #[cfg(not(feature = "dashmap"))]
    map: RwLock<HashMap<String, Arc<DomainSpecificData>>>,

    #[cfg(feature = "dashmap")]
    map: dashmap::DashMap<String, DomainSpecificData>,
}

#[cfg(feature = "dashmap")]
impl DomainsMap {
    fn get(&self, domain: &String) -> Option<Ref<String, DomainSpecificData>> {
        self.map.get(domain)
    }

    fn entry_or_insert(
        &self,
        domain: &String,
        data: DomainSpecificData,
    ) -> Ref<String, DomainSpecificData> {
        self.map.entry(domain.clone()).or_insert(data).downgrade()
    }
}

#[cfg(not(feature = "dashmap"))]
impl DomainsMap {
    fn get(&self, domain: &String) -> Option<Arc<DomainSpecificData>> {
        match self.map.read().unwrap().get(domain) {
            None => None,
            Some(data) => Some(data.clone()),
        }
    }

    fn entry_or_insert(
        &self,
        domain: &String,
        data: DomainSpecificData,
    ) -> Arc<DomainSpecificData> {
        self.map
            .write()
            .unwrap()
            .entry(domain.clone())
            .or_insert(Arc::new(data))
            .clone()
    }
}

fn domains() -> &'static DomainsMap {
    static DOMAINS: OnceLock<DomainsMap> = OnceLock::new();

    DOMAINS.get_or_init(|| DomainsMap::default())
}

/// A macro to obtain localized messages and optionally their attributes, and check the `message_id`, `attribute_id`
/// and arguments at compile time.
///
/// Compile time checks are performed using the `fallback_language`
/// specified in the current crate's `i18n.toml` confiration file.
///
/// This macro supports three different calling syntaxes which are
/// explained in the following sections.
///
/// ## No Arguments
///
/// ```ignore
/// fl!(loader: FluentLanguageLoader, "message_id")
/// ```
///
/// This is the simplest form of the `fl!()` macro, just obtaining a
/// message with no arguments. The `message_id` should be specified as
/// a literal string, and is checked at compile time.
///
/// ### Example
///
/// ```
/// use i18n_embed::{
///     fluent::{fluent_language_loader, FluentLanguageLoader},
///     LanguageLoader,
/// };
/// use i18n_embed_fl::fl;
/// use rust_embed::RustEmbed;
///
/// #[derive(RustEmbed)]
/// #[folder = "i18n/"]
/// struct Localizations;
///
/// let loader: FluentLanguageLoader = fluent_language_loader!();
/// loader
///     .load_languages(&Localizations, &[loader.fallback_language().clone()])
///     .unwrap();
///
/// // Invoke the fl!() macro to obtain the translated message, and
/// // check the message id compile time.
/// assert_eq!("Hello World!", fl!(loader, "hello-world"));
/// ```
///
/// ## Individual Arguments
///
/// ```ignore
/// fl!(
///     loader: FluentLanguageLoader,
///     "message_id",
///     arg1 = value,
///     arg2 = "value",
///     arg3 = function(),
///     ...
/// )
/// ```
///
/// This form of the `fl!()` macro allows individual arguments to be
/// specified in the form `key = value` after the `message_id`. `key`
/// needs to be a valid literal argument name, and `value` can be any
/// expression that resolves to a type that implements
/// `Into<FluentValue>`. The `key`s will be checked at compile time to
/// ensure that they match the arguments specified in original fluent
/// message.
///
/// ### Example
///
/// ```
/// # use i18n_embed::{
/// #     fluent::{fluent_language_loader, FluentLanguageLoader},
/// #     LanguageLoader,
/// # };
/// # use i18n_embed_fl::fl;
/// # use rust_embed::RustEmbed;
/// # #[derive(RustEmbed)]
/// # #[folder = "i18n/"]
/// # struct Localizations;
/// # let loader: FluentLanguageLoader = fluent_language_loader!();
/// # loader
/// #     .load_languages(&Localizations, &[loader.fallback_language().clone()])
/// #     .unwrap();
/// let calc_james = || "James".to_string();
/// pretty_assertions::assert_eq!(
///     "Hello \u{2068}Bob\u{2069} and \u{2068}James\u{2069}!",
///     // Invoke the fl!() macro to obtain the translated message, and
///     // check the message id, and arguments at compile time.
///     fl!(loader, "hello-arg-2", name1 = "Bob", name2 = calc_james())
/// );
/// ```
///
/// ## Arguments Hashmap
///
/// ```ignore
/// fl!(
///     loader: FluentLanguageLoader,
///     "message_id",
///     args: HashMap<
///         S where S: Into<Cow<'a, str>> + Clone,
///         T where T: Into<FluentValue>> + Clone>
/// )
/// ```
///
/// With this form of the `fl!()` macro, arguments can be specified at
/// runtime using a [HashMap](std::collections::HashMap), using the
/// same signature as in
/// [FluentLanguageLoader::get_args()](i18n_embed::fluent::FluentLanguageLoader::get_args()).
/// When using this method of specifying arguments, they are not
/// checked at compile time.
///
/// ### Example
///
/// ```
/// # use i18n_embed::{
/// #     fluent::{fluent_language_loader, FluentLanguageLoader},
/// #     LanguageLoader,
/// # };
/// # use i18n_embed_fl::fl;
/// # use rust_embed::RustEmbed;
/// # #[derive(RustEmbed)]
/// # #[folder = "i18n/"]
/// # struct Localizations;
/// # let loader: FluentLanguageLoader = fluent_language_loader!();
/// # loader
/// #     .load_languages(&Localizations, &[loader.fallback_language().clone()])
/// #     .unwrap();
/// use std::collections::HashMap;
///
/// let mut args: HashMap<&str, &str> = HashMap::new();
/// args.insert("name", "Bob");
///
/// assert_eq!("Hello \u{2068}Bob\u{2069}!", fl!(loader, "hello-arg", args));
/// ```
///
/// ## Attributes
///
/// In all of the above patterns you can optionally include an `attribute_id`
/// after the `message_id`, in which case `fl!` will attempt retrieving the specified
/// attribute belonging to the specified message, optionally formatted with the provided arguments.
///
/// ### Example
///
/// ```
/// # use i18n_embed::{
/// #     fluent::{fluent_language_loader, FluentLanguageLoader},
/// #     LanguageLoader,
/// # };
/// # use i18n_embed_fl::fl;
/// # use rust_embed::RustEmbed;
/// # #[derive(RustEmbed)]
/// # #[folder = "i18n/"]
/// # struct Localizations;
/// # let loader: FluentLanguageLoader = fluent_language_loader!();
/// # loader
/// #     .load_languages(&Localizations, &[loader.fallback_language().clone()])
/// #     .unwrap();
/// use std::collections::HashMap;
///
/// let mut args: HashMap<&str, &str> = HashMap::new();
/// args.insert("name", "Bob");
///
/// assert_eq!("Hello \u{2068}Bob\u{2069}'s attribute!", fl!(loader, "hello-arg", "attr", args));
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn fl(input: TokenStream) -> TokenStream {
    let input: FlMacroInput = parse_macro_input!(input as FlMacroInput);

    let fluent_loader = input.fluent_loader;
    let message_id = input.message_id;

    let domain = {
        let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
        manifest.crate_package().map(|pkg| pkg.name).unwrap_or(
            std::env::var("CARGO_PKG_NAME").expect("Error fetching `CARGO_PKG_NAME` env"),
        )
    };

    let domain_data = if let Some(domain_data) = domains().get(&domain) {
        domain_data
    } else {
        let crate_paths = i18n_config::locate_crate_paths()
            .unwrap_or_else(|error| panic!("fl!() is unable to locate crate paths: {}", error));

        let config_file_path = &crate_paths.i18n_config_file;

        let config = i18n_config::I18nConfig::from_file(config_file_path).unwrap_or_else(|err| {
            abort! {
                proc_macro2::Span::call_site(),
                format!(
                    "fl!() had a problem reading i18n config file {config_file_path:?}: {err}"
                );
                help = "Try creating the `i18n.toml` configuration file.";
            }
        });

        let fluent_config = config.fluent.unwrap_or_else(|| {
            abort! {
                proc_macro2::Span::call_site(),
                format!(
                    "fl!() had a problem parsing i18n config file {config_file_path:?}: \
                    there is no `[fluent]` subsection."
                );
                help = "Add the `[fluent]` subsection to `i18n.toml`, \
                        along with its required `assets_dir`.";
            }
        });

        // Use the domain override in the configuration.
        let domain = fluent_config.domain.unwrap_or(domain);

        let assets_dir = Path::new(&crate_paths.crate_dir).join(fluent_config.assets_dir);
        let assets = FileSystemAssets::try_new(assets_dir).unwrap();

        let fallback_language: LanguageIdentifier = config.fallback_language;

        let loader = FluentLanguageLoader::new(&domain, fallback_language.clone());

        loader
            .load_languages(&assets, &[fallback_language.clone()])
            .unwrap_or_else(|err| match err {
                i18n_embed::I18nEmbedError::LanguageNotAvailable(file, language_id) => {
                    if fallback_language != language_id {
                        panic!(
                            "fl!() encountered an unexpected problem, \
                            the language being loaded (\"{0}\") is not the \
                            `fallback_language` (\"{1}\")",
                            language_id, fallback_language
                        )
                    }
                    abort! {
                        proc_macro2::Span::call_site(),
                        format!(
                            "fl!() was unable to load the localization \
                            file for the `fallback_language` \
                            (\"{fallback_language}\"): {file}"
                        );
                        help = "Try creating the required fluent localization file.";
                    }
                }
                _ => panic!(
                    "fl!() had an unexpected problem while \
                        loading language \"{0}\": {1}",
                    fallback_language, err
                ),
            });

        let data = DomainSpecificData {
            loader,
            _assets: assets,
        };

        domains().entry_or_insert(&domain, data)
    };

    let message_id_string = match &message_id {
        syn::Lit::Str(message_id_str) => {
            let message_id_str = message_id_str.value();
            Some(message_id_str)
        }
        unexpected_lit => {
            emit_error! {
                unexpected_lit,
                "fl!() `message_id` should be a literal rust string"
            };
            None
        }
    };

    let attr = input.attr;
    let attr_str;
    let attr_lit = match &attr {
        FlAttr::Attr(literal) => match literal {
            syn::Lit::Str(string_lit) => {
                attr_str = Some(string_lit.value());
                Some(literal)
            }
            unexpected_lit => {
                attr_str = None;
                emit_error! {
                    unexpected_lit,
                    "fl!() `message_id` should be a literal rust string"
                };
                None
            }
        },
        FlAttr::None => {
            attr_str = None;
            None
        }
    };

    // If we have already confirmed that the loader has the message.
    // `false` if we haven't checked, or we have checked but no
    // message was found.
    let mut checked_loader_has_message = false;
    // Same procedure for attributes
    let mut checked_message_has_attribute = false;

    let gen = match input.args {
        FlArgs::HashMap(args_hash_map) => {
            if attr_lit.is_none() {
                quote! {
                    (#fluent_loader).get_args(#message_id, #args_hash_map)
                }
            } else {
                quote! {
                    (#fluent_loader).get_attr_args(#message_id, #attr_lit, #args_hash_map)
                }
            }
        }
        FlArgs::None => {
            if attr_lit.is_none() {
                quote! {
                    (#fluent_loader).get(#message_id)
                }
            } else {
                quote! {
                    (#fluent_loader).get_attr(#message_id, #attr_lit)
                }
            }
        }
        FlArgs::KeyValuePairs { specified_args } => {
            let mut arg_assignments = proc_macro2::TokenStream::default();
            for (key, value) in &specified_args {
                arg_assignments = quote! {
                    #arg_assignments
                    args.insert(#key, #value.into());
                }
            }

            if attr_lit.is_none() {
                if let Some(message_id_str) = &message_id_string {
                    checked_loader_has_message = domain_data
                        .loader
                        .with_fluent_message_and_bundle(message_id_str, |message, bundle| {
                            check_message_args(message, bundle, &specified_args);
                        })
                        .is_some();
                }

                let gen = quote! {
                    (#fluent_loader).get_args_concrete(
                        #message_id,
                        {
                            let mut args = std::collections::HashMap::new();
                            #arg_assignments
                            args
                        })
                };

                gen
            } else {
                if let Some(message_id_str) = &message_id_string {
                    if let Some(attr_id_str) = &attr_str {
                        let attr_res = domain_data.loader.with_fluent_message_and_bundle(
                            message_id_str,
                            |message, bundle| match message.get_attribute(attr_id_str) {
                                Some(attr) => {
                                    check_attribute_args(attr, bundle, &specified_args);
                                    true
                                }
                                None => false,
                            },
                        );
                        checked_loader_has_message = attr_res.is_some();
                        checked_message_has_attribute = attr_res.unwrap_or(false);
                    }
                }

                let gen = quote! {
                    (#fluent_loader).get_attr_args_concrete(
                        #message_id,
                        #attr_lit,
                        {
                            let mut args = std::collections::HashMap::new();
                            #arg_assignments
                            args
                        })
                };

                gen
            }
        }
    };

    if let Some(message_id_str) = &message_id_string {
        if !checked_loader_has_message && !domain_data.loader.has(message_id_str) {
            let suggestions =
                fuzzy_message_suggestions(&domain_data.loader, message_id_str, 5).join("\n");

            let hint = format!(
                "Perhaps you are looking for one of the following messages?\n\n\
                {suggestions}"
            );

            emit_error! {
                message_id,
                format!(
                    "fl!() `message_id` validation failed. `message_id` \
                    of \"{0}\" does not exist in the `fallback_language` (\"{1}\")",
                    message_id_str,
                    domain_data.loader.current_language(),
                );
                help = "Enter the correct `message_id` or create \
                        the message in the localization file if the \
                        intended message does not yet exist.";

                hint = hint;
            };
        } else if let Some(attr_id_str) = &attr_str {
            if !checked_message_has_attribute
                && !&domain_data.loader.has_attr(message_id_str, attr_id_str)
            {
                let suggestions = &domain_data
                    .loader
                    .with_fluent_message(message_id_str, |message| {
                        fuzzy_attribute_suggestions(&message, attr_id_str, 5).join("\n")
                    })
                    .unwrap();

                let hint = format!(
                    "Perhaps you are looking for one of the following attributes?\n\n\
                    {suggestions}"
                );

                emit_error! {
                    attr_lit,
                    format!(
                        "fl!() `attribute_id` validation failed. `attribute_id` \
                        of \"{0}\" does not exist in the `fallback_language` (\"{1}\")",
                        attr_id_str,
                        domain_data.loader.current_language(),
                    );
                    help = "Enter the correct `attribute_id` or create \
                            the attribute associated with the message in the localization file if the \
                            intended attribute does not yet exist.";

                    hint = hint;
                };
            }
        }
    }

    gen.into()
}

fn fuzzy_message_suggestions(
    loader: &FluentLanguageLoader,
    message_id_str: &str,
    n_suggestions: usize,
) -> Vec<String> {
    let mut scored_messages: Vec<(String, usize)> =
        loader.with_message_iter(loader.fallback_language(), |message_iter| {
            message_iter
                .map(|message| {
                    (
                        message.id.name.to_string(),
                        strsim::levenshtein(message_id_str, message.id.name),
                    )
                })
                .collect()
        });

    scored_messages.sort_by_key(|(_message, score)| *score);

    scored_messages.truncate(n_suggestions);

    scored_messages
        .into_iter()
        .map(|(message, _score)| message)
        .collect()
}

fn fuzzy_attribute_suggestions(
    message: &FluentMessage<'_>,
    attribute_id_str: &str,
    n_suggestions: usize,
) -> Vec<String> {
    let mut scored_attributes: Vec<(String, usize)> = message
        .attributes()
        .map(|attribute| {
            (
                attribute.id().to_string(),
                strsim::levenshtein(attribute_id_str, attribute.id()),
            )
        })
        .collect();

    scored_attributes.sort_by_key(|(_attr, score)| *score);

    scored_attributes.truncate(n_suggestions);

    scored_attributes
        .into_iter()
        .map(|(attribute, _score)| attribute)
        .collect()
}

fn check_message_args<R>(
    message: FluentMessage<'_>,
    bundle: &FluentBundle<R>,
    specified_args: &Vec<(syn::LitStr, Box<syn::Expr>)>,
) where
    R: std::borrow::Borrow<FluentResource>,
{
    if let Some(pattern) = message.value() {
        let mut args = Vec::new();
        args_from_pattern(pattern, bundle, &mut args);

        let args_set: HashSet<&str> = args.into_iter().collect();

        let key_args: Vec<String> = specified_args
            .iter()
            .map(|(key, _value)| {
                let arg = key.value();

                if !args_set.contains(arg.as_str()) {
                    let available_args: String = args_set
                        .iter()
                        .map(|arg| format!("`{arg}`"))
                        .collect::<Vec<String>>()
                        .join(", ");

                    emit_error! {
                        key,
                        format!(
                            "fl!() argument `{0}` does not exist in the \
                            fluent message. Available arguments: {1}.",
                            &arg, available_args
                        );
                        help = "Enter the correct arguments, or fix the message \
                                in the fluent localization file so that the arguments \
                                match this macro invocation.";
                    };
                }

                arg
            })
            .collect();

        let key_args_set: HashSet<&str> = key_args.iter().map(|v| v.as_str()).collect();

        let unspecified_args: Vec<String> = args_set
            .iter()
            .filter_map(|arg| {
                if !key_args_set.contains(arg) {
                    Some(format!("`{arg}`"))
                } else {
                    None
                }
            })
            .collect();

        if !unspecified_args.is_empty() {
            emit_error! {
                proc_macro2::Span::call_site(),
                format!(
                    "fl!() the following arguments have not been specified: {}",
                    unspecified_args.join(", ")
                );
                help = "Enter the correct arguments, or fix the message \
                        in the fluent localization file so that the arguments \
                        match this macro invocation.";
            };
        }
    }
}

fn check_attribute_args<R>(
    attr: FluentAttribute<'_>,
    bundle: &FluentBundle<R>,
    specified_args: &Vec<(syn::LitStr, Box<syn::Expr>)>,
) where
    R: std::borrow::Borrow<FluentResource>,
{
    let pattern = attr.value();
    let mut args = Vec::new();
    args_from_pattern(pattern, bundle, &mut args);

    let args_set: HashSet<&str> = args.into_iter().collect();

    let key_args: Vec<String> = specified_args
        .iter()
        .map(|(key, _value)| {
            let arg = key.value();

            if !args_set.contains(arg.as_str()) {
                let available_args: String = args_set
                    .iter()
                    .map(|arg| format!("`{arg}`"))
                    .collect::<Vec<String>>()
                    .join(", ");

                emit_error! {
                    key,
                    format!(
                        "fl!() argument `{0}` does not exist in the \
                        fluent attribute. Available arguments: {1}.",
                        &arg, available_args
                    );
                    help = "Enter the correct arguments, or fix the attribute \
                            in the fluent localization file so that the arguments \
                            match this macro invocation.";
                };
            }

            arg
        })
        .collect();

    let key_args_set: HashSet<&str> = key_args.iter().map(|v| v.as_str()).collect();

    let unspecified_args: Vec<String> = args_set
        .iter()
        .filter_map(|arg| {
            if !key_args_set.contains(arg) {
                Some(format!("`{arg}`"))
            } else {
                None
            }
        })
        .collect();

    if !unspecified_args.is_empty() {
        emit_error! {
            proc_macro2::Span::call_site(),
            format!(
                "fl!() the following arguments have not been specified: {}",
                unspecified_args.join(", ")
            );
            help = "Enter the correct arguments, or fix the attribute \
                    in the fluent localization file so that the arguments \
                    match this macro invocation.";
        };
    }
}

fn args_from_pattern<'m, R>(
    pattern: &Pattern<&'m str>,
    bundle: &'m FluentBundle<R>,
    args: &mut Vec<&'m str>,
) where
    R: std::borrow::Borrow<FluentResource>,
{
    pattern.elements.iter().for_each(|element| {
        if let PatternElement::Placeable { expression } = element {
            args_from_expression(expression, bundle, args)
        }
    });
}

fn args_from_expression<'m, R>(
    expr: &Expression<&'m str>,
    bundle: &'m FluentBundle<R>,
    args: &mut Vec<&'m str>,
) where
    R: std::borrow::Borrow<FluentResource>,
{
    match expr {
        Expression::Inline(inline_expr) => {
            args_from_inline_expression(inline_expr, bundle, args);
        }
        Expression::Select { selector, variants } => {
            args_from_inline_expression(selector, bundle, args);

            variants.iter().for_each(|variant| {
                args_from_pattern(&variant.value, bundle, args);
            })
        }
    }
}

fn args_from_inline_expression<'m, R>(
    inline_expr: &InlineExpression<&'m str>,
    bundle: &'m FluentBundle<R>,
    args: &mut Vec<&'m str>,
) where
    R: std::borrow::Borrow<FluentResource>,
{
    match inline_expr {
        InlineExpression::FunctionReference {
            id: _,
            arguments: call_args,
        } => {
            args_from_call_arguments(call_args, bundle, args);
        }
        InlineExpression::TermReference {
            id: _,
            attribute: _,
            arguments: Some(call_args),
        } => {
            args_from_call_arguments(call_args, bundle, args);
        }
        InlineExpression::VariableReference { id } => args.push(id.name),
        InlineExpression::Placeable { expression } => {
            args_from_expression(expression, bundle, args)
        }
        InlineExpression::MessageReference {
            id,
            attribute: None,
        } => {
            bundle
                .get_message(&id.name)
                .and_then(|m| m.value())
                .map(|p| args_from_pattern(p, bundle, args));
        }
        InlineExpression::MessageReference {
            id,
            attribute: Some(attribute),
        } => {
            bundle
                .get_message(&id.name)
                .and_then(|m| m.get_attribute(&attribute.name))
                .map(|m| m.value())
                .map(|p| args_from_pattern(p, bundle, args));
        }
        _ => {}
    }
}

fn args_from_call_arguments<'m, R>(
    call_args: &CallArguments<&'m str>,
    bundle: &'m FluentBundle<R>,
    args: &mut Vec<&'m str>,
) where
    R: std::borrow::Borrow<FluentResource>,
{
    call_args.positional.iter().for_each(|expr| {
        args_from_inline_expression(expr, bundle, args);
    });

    call_args.named.iter().for_each(|named_arg| {
        args_from_inline_expression(&named_arg.value, bundle, args);
    })
}
