#![feature(proc_macro_diagnostic)]

use fluent::FluentMessage;
use fluent_syntax::ast::{CallArguments, Expression, InlineExpression, Pattern, PatternElement};
use i18n_embed::{fluent::FluentLanguageLoader, FileSystemAssets, LanguageLoader};
use proc_macro::TokenStream;
use quote::quote;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use syn::{parse::Parse, parse_macro_input, spanned::Spanned};
use unic_langid::LanguageIdentifier;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");

#[derive(Debug)]
enum FlArgs {
    /// `fl!(LOADER, "message", args)` where `args` is a
    /// `HashMap<&'a str, FluentValue<'a>>`.
    HashMap(syn::Ident),
    /// ```ignore
    /// fl!(LOADER, "message",
    ///     arg1 = "value",
    ///     arg2 = value2,
    ///     arg3 = calc_value());
    /// ```
    KeyValuePairs {
        specified_args: HashMap<syn::LitStr, Box<syn::Expr>>,
    },
    /// `fl!(LOADER, "message")` no arguments after the message id.
    None,
}

impl Parse for FlArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;

            let lookahead = input.fork();
            if lookahead.parse::<syn::Ident>().is_ok() && lookahead.is_empty() {
                let hash_map = input.parse()?;
                return Ok(FlArgs::HashMap(hash_map));
            }

            let mut args_map: HashMap<syn::LitStr, Box<syn::Expr>> = HashMap::new();

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

                if let Some(_duplicate) =
                    args_map.insert(argument_name_lit_str.clone(), argument_value)
                {
                    return Err(syn::Error::new(
                        argument_name_lit_str.span(),
                        format!(
                            "fl!() macro contains a duplicate argument `{}`",
                            argument_name_lit_str.value()
                        ),
                    ));
                }

                // parse the next comma if there is one
                let _result = input.parse::<syn::Token![,]>();
            }

            if args_map.is_empty() {
                let span = match input.fork().parse::<syn::Expr>() {
                    Ok(expr) => expr.span(),
                    Err(_) => input.span(),
                };
                Err(syn::Error::new(span, "fl!() unable to parse args input"))
            } else {
                Ok(FlArgs::KeyValuePairs {
                    specified_args: args_map,
                })
            }
        } else {
            Ok(FlArgs::None)
        }
    }
}

/// Input for the [fl()] macro.
struct FlMacroInput {
    fluent_loader: syn::Ident,
    message_id: syn::Lit,
    args: FlArgs,
}

impl Parse for FlMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fluent_loader = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let message_id = input.parse()?;

        let args = input.parse()?;

        Ok(Self {
            fluent_loader,
            message_id,
            args,
        })
    }
}

struct DomainSpecificData {
    loader: FluentLanguageLoader,
    _assets: FileSystemAssets,
}

lazy_static::lazy_static! {
    /// Cached data specific to each localization domain, to improve
    /// performance of subsequent macro invokations.
    static ref DOMAINS: dashmap::DashMap<String, DomainSpecificData> =
        dashmap::DashMap::new();
}

/// A macro to obtain localized messages, and check the `message_id`
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
///     .load_languages(&Localizations, &[loader.fallback_language()])
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
/// #     .load_languages(&Localizations, &[loader.fallback_language()])
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
/// When using this method of specifying argments, they are not
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
/// #     .load_languages(&Localizations, &[loader.fallback_language()])
/// #     .unwrap();
/// use std::collections::HashMap;
///
/// let mut args: HashMap<&str, &str> = HashMap::new();
/// args.insert("name", "Bob");
///
/// assert_eq!("Hello \u{2068}Bob\u{2069}!", fl!(loader, "hello-arg", args));
/// ```
#[proc_macro]
pub fn fl(input: TokenStream) -> TokenStream {
    let input: FlMacroInput = parse_macro_input!(input as FlMacroInput);

    let fluent_loader = input.fluent_loader;
    let message_id = input.message_id;

    let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");

    let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");
    let domain = current_crate_package.name;

    let domain_data = if let Some(domain_data) = DOMAINS.get(&domain) {
        domain_data
    } else {
        let crate_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_else(|| {
            panic!(
                "fl!() had a problem reading `CARGO_MANIFEST_DIR` \
            environment variable"
            )
        });
        let config_file_path = Path::new(&crate_dir).join("i18n.toml");

        let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
            panic!(
                "fl!() had a problem reading config file {0:?}: {1}",
                config_file_path, err
            )
        });

        let fluent_config = config.fluent.unwrap_or_else(|| {
            panic!(
                "fl!() had a problem parsing config file {0:?}: \
                there is no `[fluent]` subsection.",
                config_file_path
            )
        });

        let assets_dir = Path::new(&crate_dir).join(fluent_config.assets_dir);
        let assets = FileSystemAssets::new(assets_dir);

        let fallback_language: LanguageIdentifier = config.fallback_language;

        let loader = FluentLanguageLoader::new(&domain, fallback_language.clone());

        loader
            .load_languages(&assets, &[&fallback_language])
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
                    panic!(
                        "fl!() was unable to load the localization \
                        file for the `fallback_language` (\"{0}\"): {1}",
                        fallback_language, file,
                    )
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

        DOMAINS.insert_and_get(domain.clone(), data)
    };

    let message_id_string = match &message_id {
        syn::Lit::Str(message_id_str) => {
            let message_id_str = message_id_str.value();
            Some(message_id_str)
        }
        unexpected_lit => {
            unexpected_lit
                .span()
                .unstable()
                .error("fl!() `message_id` should be a &'static str")
                .emit();
            None
        }
    };

    // If we have already confirmed that the loader has the message.
    // `false` if we haven't checked, or we have checked but no
    // message was found.
    let mut checked_loader_has_message = false;

    let gen = match input.args {
        FlArgs::HashMap(args_hash_map) => {
            quote! {
                #fluent_loader.get_args(#message_id, #args_hash_map)
            }
        }
        FlArgs::None => {
            quote! {
                #fluent_loader.get(#message_id)
            }
        }
        FlArgs::KeyValuePairs { specified_args } => {
            let mut arg_assignments = proc_macro2::TokenStream::default();

            if let Some(message_id_str) = &message_id_string {
                checked_loader_has_message = domain_data
                    .loader
                    .with_fluent_message(message_id_str, |message: FluentMessage<'_>| {
                        check_message_args(message, &specified_args);
                    })
                    .is_some();
            }

            for (key, value) in &specified_args {
                arg_assignments = quote! {
                    #arg_assignments
                    args.insert(#key, #value.into());
                }
            }

            let gen = quote! {
                #fluent_loader.get_args_concrete(
                    #message_id,
                    {
                        let mut args = std::collections::HashMap::new();
                        #arg_assignments
                        args
                    })
            };

            if let Some(message_id_str) = &message_id_string {
                if !checked_loader_has_message && !domain_data.loader.has(&message_id_str) {
                    message_id
                        .span()
                        .unstable()
                        .error(&format!(
                            "fl!() `message_id` validation failed. `message_id` \
                            of \"{0}\" does not exist in the `fallback_language` (\"{1}\")",
                            message_id_str,
                            domain_data.loader.current_language(),
                        ))
                        .emit();
                }
            }

            gen
        }
    };

    gen.into()
}

fn check_message_args<'a>(
    message: FluentMessage<'a>,
    specified_args: &HashMap<syn::LitStr, Box<syn::Expr>>,
) {
    if let Some(pattern) = message.value {
        let mut args = Vec::new();
        args_from_pattern(pattern, &mut args);

        let args_set: HashSet<&str> = args.into_iter().collect();

        let key_args: Vec<String> = specified_args
            .keys()
            .into_iter()
            .map(|key| {
                let arg = key.value();

                if !args_set.contains(arg.as_str()) {
                    let available_args: String = args_set
                        .iter()
                        .map(|arg| format!("`{}`", arg))
                        .collect::<Vec<String>>()
                        .join(", ");

                    key.span()
                        .unstable()
                        .error(format!(
                            "fl!() argument `{0}` does not exist in the \
                            fluent message. Available arguments: {1}.",
                            &arg, available_args
                        ))
                        .emit();
                }

                arg
            })
            .collect();

        let key_args_set: HashSet<&str> = key_args.iter().map(|v| v.as_str()).collect();

        let unspecified_args: Vec<String> = args_set
            .iter()
            .filter_map(|arg| {
                if !key_args_set.contains(arg) {
                    Some(format!("`{}`", arg))
                } else {
                    None
                }
            })
            .collect();

        if !unspecified_args.is_empty() {
            proc_macro2::Span::call_site()
                .unstable()
                .error(format!(
                    "fl!() the following arguments have not been specified: {}",
                    unspecified_args.join(", ")
                ))
                .emit();
        }
    }
}

fn args_from_pattern<'a>(pattern: &'a Pattern, args: &mut Vec<&'a str>) {
    pattern.elements.iter().for_each(|element| {
        if let PatternElement::Placeable(expr) = element {
            args_from_expression(expr, args)
        }
    });
}

fn args_from_expression<'a>(expr: &'a Expression, args: &mut Vec<&'a str>) {
    match expr {
        Expression::InlineExpression(inline_expr) => {
            args_from_inline_expression(inline_expr, args);
        }
        Expression::SelectExpression { selector, variants } => {
            args_from_inline_expression(selector, args);

            variants.iter().for_each(|variant| {
                args_from_pattern(&variant.value, args);
            })
        }
    }
}

fn args_from_inline_expression<'a>(inline_expr: &'a InlineExpression, args: &mut Vec<&'a str>) {
    match inline_expr {
        InlineExpression::FunctionReference { id: _, arguments } => {
            if let Some(call_args) = arguments {
                args_from_call_arguments(call_args, args);
            }
        }
        InlineExpression::TermReference {
            id: _,
            attribute: _,
            arguments,
        } => {
            if let Some(call_args) = arguments {
                args_from_call_arguments(call_args, args);
            }
        }
        InlineExpression::VariableReference { id } => args.push(id.name),
        InlineExpression::Placeable { expression } => args_from_expression(expression, args),
        _ => {}
    }
}

fn args_from_call_arguments<'a>(call_args: &'a CallArguments, args: &mut Vec<&'a str>) {
    call_args.positional.iter().for_each(|expr| {
        args_from_inline_expression(expr, args);
    });

    call_args.named.iter().for_each(|named_arg| {
        args_from_inline_expression(&named_arg.value, args);
    })
}
