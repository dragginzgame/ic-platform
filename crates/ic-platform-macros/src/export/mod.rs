// Derived from ic-cdk-macros 0.20.3; see README.md and LICENSE.
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::fmt::Formatter;
use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Error, ItemFn, Path, Signature, parse_str};

pub(super) struct ExportAttributes<P = String> {
    pub name: Option<String>,
    pub guard: Vec<P>,
    /// The name of the function to use for decoding arguments.
    /// Required for callable entries, including unit requests.
    ///
    /// Even if the argument type is empty, the specified function will still be executed.
    pub decode_with: Option<P>,
    /// The name of the function to use for encoding the return value.
    /// Required for automatic replies. No default protocol is selected.
    ///
    /// If the method returns a tuple, this custom encoder function should take the tuple as an argument.
    pub encode_with: Option<P>,
    pub manual_reply: bool,
    pub composite: bool,
    pub cratename: Option<P>,
}

impl<P> Default for ExportAttributes<P> {
    fn default() -> Self {
        Self {
            name: None,
            guard: Vec::new(),
            decode_with: None,
            encode_with: None,
            manual_reply: false,
            composite: false,
            cratename: None,
        }
    }
}

impl ExportAttributes {
    fn parse(tokens: TokenStream) -> syn::Result<Self> {
        let mut attrs = Self::default();
        let mut seen = std::collections::BTreeSet::new();
        syn::meta::parser(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(meta.error("expected a single entrypoint option name"));
            };
            let name = ident.to_string();
            if name == "hidden" {
                return Err(meta.error("hidden is retired; remove it (it had no effect)"));
            }
            if name != "guard" && !seen.insert(name.clone()) {
                return Err(meta.error(format!("duplicate entrypoint option `{name}`")));
            }
            match name.as_str() {
                "name" | "guard" | "decode_with" | "encode_with" | "crate" => {
                    let value = meta.value()?.parse::<syn::LitStr>()?.value();
                    match name.as_str() {
                        "name" => attrs.name = Some(value),
                        "guard" => attrs.guard.push(value),
                        "decode_with" => attrs.decode_with = Some(value),
                        "encode_with" => attrs.encode_with = Some(value),
                        "crate" => attrs.cratename = Some(value),
                        _ => unreachable!(),
                    }
                }
                "manual_reply" | "composite" => {
                    let value = if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<syn::LitBool>()?.value
                    } else {
                        true
                    };
                    if name == "manual_reply" {
                        attrs.manual_reply = value;
                    } else {
                        attrs.composite = value;
                    }
                }
                _ => return Err(meta.error(format!("unknown entrypoint option `{name}`"))),
            }
            Ok(())
        })
        .parse2(tokens)?;
        Ok(attrs)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MethodType {
    Init,
    PreUpgrade,
    PostUpgrade,
    Update,
    Query,
    Heartbeat,
    InspectMessage,
    OnLowWasmMemory,
}

struct MethodInfo {
    name: &'static str,
    lifecycle: bool,
    arguments: bool,
    persistent: bool,
}

impl MethodType {
    fn info(self) -> MethodInfo {
        let (name, lifecycle, arguments, persistent) = match self {
            Self::Init => ("init", true, true, true),
            Self::PreUpgrade => ("pre_upgrade", true, false, true),
            Self::PostUpgrade => ("post_upgrade", true, true, true),
            Self::Update => ("update", false, true, true),
            Self::Query => ("query", false, true, false),
            Self::Heartbeat => ("heartbeat", true, false, true),
            Self::InspectMessage => ("inspect_message", true, false, false),
            Self::OnLowWasmMemory => ("on_low_wasm_memory", true, false, true),
        };
        MethodInfo {
            name,
            lifecycle,
            arguments,
            persistent,
        }
    }

    pub fn is_lifecycle(&self) -> bool {
        self.info().lifecycle
    }

    pub fn can_have_args(&self) -> bool {
        self.info().arguments
    }

    pub fn is_state_persistent(&self) -> bool {
        self.info().persistent
    }
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.info().name)
    }
}

fn get_args(method: MethodType, signature: &Signature) -> Result<Vec<Ident>, Error> {
    crate::signature::argument_types(signature, |r| {
        Error::new(
            r.span(),
            format!("#[{method}] cannot be above functions with `self` as a parameter."),
        )
    })
    .map(|types| {
        types
            .iter()
            .enumerate()
            .map(|(i, _)| format_ident!("__platform_arg_{i}", span = Span::mixed_site()))
            .collect()
    })
}

// Native object formats need portable symbols, but replacing punctuation can
// merge distinct IC export names. Hex encodes every UTF-8 byte without loss.
fn host_export_name(export_name: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut symbol = String::with_capacity(12 + export_name.len() * 2);
    symbol.push_str("__ic_export_");
    for byte in export_name.bytes() {
        symbol.push(HEX[usize::from(byte >> 4)] as char);
        symbol.push(HEX[usize::from(byte & 15)] as char);
    }
    symbol
}

enum Reply {
    None,
    Automatic { encoder: Path, takes_result: bool },
}

/// Validated entrypoint decisions. Emission cannot select a codec or executor.
struct EntryPlan {
    item: TokenStream,
    name: Ident,
    outer_function_ident: Ident,
    export_name: String,
    cratename: Path,
    guards: Vec<Path>,
    arg_tuple: Vec<Ident>,
    decoder: Option<Path>,
    reply: Reply,
    asynchronous: bool,
    context: Ident,
}

impl EntryPlan {
    fn parse(method: MethodType, attr: TokenStream, item: TokenStream) -> Result<Self, Error> {
        let attr_span = attr.span();
        let attrs = ExportAttributes::parse(attr)?;
        Self::build(method, attrs, attr_span, item, None, |path| {
            parse_str(&path)
        })
    }

    fn build<P>(
        method: MethodType,
        attrs: ExportAttributes<P>,
        attr_span: Span,
        item: TokenStream,
        parsed: Option<ItemFn>,
        parse_path: impl Fn(P) -> Result<Path, Error>,
    ) -> Result<Self, Error> {
        if attrs.composite && method != MethodType::Query {
            return Err(Error::new(
                attr_span,
                "composite is only supported on query methods",
            ));
        }
        if attrs.manual_reply && method.is_lifecycle() {
            return Err(Error::new(
                attr_span,
                "manual_reply is not supported on lifecycle hooks",
            ));
        }
        let fun = if let Some(fun) = parsed {
            fun
        } else {
            syn::parse2::<syn::ItemFn>(item.clone()).map_err(|e| {
                Error::new(
                    item.span(),
                    format!("#[{method}] must be above a function. \n{e}"),
                )
            })?
        };
        let signature = &fun.sig;
        let generics = &signature.generics;

        if !generics.params.is_empty() {
            return Err(Error::new(
                generics.span(),
                format!("#[{method}] must be above a function with no generic parameters."),
            ));
        }
        let cratename = attrs
            .cratename
            .map(&parse_path)
            .transpose()?
            .unwrap_or_else(|| syn::parse_quote!(::ic_platform::cdk));

        // Function and export names.
        let name = &signature.ident;
        let plain_name = name.unraw();
        let outer_function_ident = format_ident!("__canister_method_{plain_name}");
        let function_name = if let Some(custom_name) = attrs.name {
            if method.is_lifecycle() {
                return Err(Error::new(
                    attr_span,
                    format!("#[{method}] cannot have a custom name"),
                ));
            }
            if custom_name.starts_with("<ic-cdk internal>") {
                return Err(Error::new(
                    attr_span,
                    "Functions starting with `<ic-cdk internal>` are reserved for CDK internal use.",
                ));
            }
            custom_name
        } else {
            plain_name.to_string()
        };
        let export_name = if method.is_lifecycle() {
            format!("canister_{method}")
        } else if method == MethodType::Query && attrs.composite {
            format!("canister_composite_query {function_name}",)
        } else {
            format!("canister_{method} {function_name}")
        };

        // Guards remain ordered; parse paths before argument admission.
        if !attrs.guard.is_empty() && method.is_lifecycle() {
            return Err(Error::new(
                attr_span,
                format!("#[{method}] cannot have guard function(s)."),
            ));
        }
        let guards = attrs
            .guard
            .into_iter()
            .map(&parse_path)
            .collect::<Result<Vec<_>, Error>>()?;

        // Argument admission, including explicit decoding for unit requests.
        let arg_tuple = get_args(method, signature)?;
        if !method.can_have_args() {
            if !arg_tuple.is_empty() {
                return Err(Error::new(
                    Span::call_site(),
                    format!("#[{method}] function cannot have arguments."),
                ));
            }
            if attrs.decode_with.is_some() {
                return Err(Error::new(
                    attr_span,
                    format!("#[{method}] function cannot have a decode_with attribute."),
                ));
            }
        }
        let decoder = if let Some(decode_with) = attrs.decode_with {
            Some(parse_path(decode_with)?)
        } else if method.can_have_args() && (!method.is_lifecycle() || !arg_tuple.is_empty()) {
            return Err(Error::new(
                attr_span,
                "endpoint requires decode_with, including unit RPC requests; use the consumer-owned Wire decoder",
            ));
        } else {
            None
        };

        // Reply admission.
        let return_length = crate::signature::ReturnShape::of(signature).arity();
        if method.is_lifecycle() {
            if return_length > 0 {
                return Err(Error::new(
                    Span::call_site(),
                    format!("#[{method}] function cannot have a return value."),
                ));
            }
            if attrs.encode_with.is_some() {
                return Err(Error::new(
                    attr_span,
                    format!("#[{method}] function cannot have an encode_with attribute."),
                ));
            }
        }
        let reply = if method.is_lifecycle() || attrs.manual_reply {
            Reply::None
        } else if let Some(encode_with) = attrs.encode_with {
            Reply::Automatic {
                encoder: parse_path(encode_with)?,
                takes_result: return_length != 0,
            }
        } else {
            return Err(Error::new(
                attr_span,
                "endpoint requires encode_with or manual_reply; no default Candid codec is supplied",
            ));
        };
        let context = if method.is_state_persistent() {
            format_ident!("in_executor_context")
        } else {
            format_ident!("in_query_executor_context")
        };
        Ok(Self {
            name: name.clone(),
            outer_function_ident,
            export_name,
            cratename,
            guards,
            arg_tuple,
            decoder,
            reply,
            context,
            asynchronous: signature.asyncness.is_some(),
            item,
        })
    }

    fn expand(self) -> TokenStream {
        let Self {
            item,
            name,
            outer_function_ident,
            export_name,
            cratename,
            guards,
            arg_tuple,
            decoder,
            reply,
            asynchronous,
            context: async_context_name,
        } = self;
        // User-supplied function/codec paths retain their own spans. Generated locals
        // must not capture those paths, even when they have the same spelling.
        let guard_result = Ident::new("r", Span::mixed_site());
        let argument_bytes = Ident::new("arg_bytes", Span::mixed_site());
        let result = Ident::new("result", Span::mixed_site());
        let host_compatible_name = host_export_name(&export_name);
        let guards = guards.iter().map(|guard_path| quote! {
            let #guard_result: ::std::result::Result<(), ::std::string::String> = #guard_path ();
            if let ::std::result::Result::Err(e) = #guard_result {
                #cratename::api::msg_reject(&e);
                return;
            }
        });
        let guard = quote! { #(#guards)* };
        let arg_decode = if let Some(decoder) = decoder {
            if arg_tuple.len() == 1 {
                let arg_one = &arg_tuple[0];
                quote! {
                    let #argument_bytes = #cratename::api::msg_arg_data();
                    let #arg_one = #decoder(#argument_bytes);
                }
            } else {
                quote! {
                    let #argument_bytes = #cratename::api::msg_arg_data();
                    let ( #( #arg_tuple, )* ) = #decoder(#argument_bytes);
                }
            }
        } else {
            quote! {}
        };
        let function_call = if asynchronous {
            quote! { #name ( #(#arg_tuple),* ) .await }
        } else {
            quote! { #name ( #(#arg_tuple),* ) }
        };
        let return_encode = match reply {
            Reply::None => quote! {},
            Reply::Automatic {
                encoder,
                takes_result,
            } => {
                let return_bytes = if takes_result {
                    quote! { #encoder(#result) }
                } else {
                    quote! { #encoder() }
                };
                quote! {
                    let bytes: ::std::vec::Vec<u8> = #return_bytes;
                    #cratename::api::msg_reply(bytes);
                }
            }
        };
        // Codec-defined endpoints are never registered as Candid services.
        let body = if asynchronous {
            quote! {
                #cratename::futures::internals::#async_context_name(|| {
                    #guard
                    #[allow(clippy::disallowed_methods)]
                    #cratename::futures::spawn(async {
                        #arg_decode
                        let #result = #function_call;
                        #return_encode
                    });
                });
            }
        } else {
            quote! {
                #guard
                #cratename::futures::internals::#async_context_name(|| {
                    #arg_decode
                    let #result = #function_call;
                    #return_encode
                });
            }
        };

        quote! {
            #[cfg_attr(target_family = "wasm", unsafe(export_name = #export_name))]
            #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = #host_compatible_name))]
            fn #outer_function_ident() {
                #body
            }


            #item
        }
    }
}

/// Internal typed bridge; the normal attribute parser remains the public grammar.
pub(super) fn external_entry(
    query: bool,
    attrs: ExportAttributes<Path>,
    function: ItemFn,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    let method = if query {
        MethodType::Query
    } else {
        MethodType::Update
    };
    Ok(EntryPlan::build(method, attrs, item.span(), item, Some(function), Ok)?.expand())
}

fn dfn_macro(
    method: MethodType,
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    Ok(EntryPlan::parse(method, attr, item)?.expand())
}

pub(crate) fn ic_query(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Query, attr, item)
}

pub(crate) fn ic_update(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Update, attr, item)
}

pub(crate) fn ic_init(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Init, attr, item)
}

pub(crate) fn ic_pre_upgrade(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::PreUpgrade, attr, item)
}

pub(crate) fn ic_post_upgrade(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::PostUpgrade, attr, item)
}

pub(crate) fn ic_heartbeat(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Heartbeat, attr, item)
}

pub(crate) fn ic_inspect_message(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::InspectMessage, attr, item)
}

pub(crate) fn ic_on_low_wasm_memory(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::OnLowWasmMemory, attr, item)
}

#[cfg(test)]
mod tests;
