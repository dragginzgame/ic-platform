// Derived from ic-cdk-macros 0.20.3; see README.md and LICENSE.
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::fmt::Formatter;
use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, Path, ReturnType, Signature, Type, parse_str};

#[derive(Default)]
struct ExportAttributes {
    pub name: Option<String>,
    pub guard: Vec<String>,
    /// The name of the function to use for decoding arguments.
    /// Required for callable entries, including unit requests.
    ///
    /// Even if the argument type is empty, the specified function will still be executed.
    pub decode_with: Option<String>,
    /// The name of the function to use for encoding the return value.
    /// Required for automatic replies. No default protocol is selected.
    ///
    /// If the method returns a tuple, this custom encoder function should take the tuple as an argument.
    pub encode_with: Option<String>,
    pub manual_reply: bool,
    pub composite: bool,
    pub cratename: Option<String>,
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

impl MethodType {
    /// A lifecycle method is a method that is called by the system and not by the user.
    /// So far, `update` and `query` are the only methods that are not lifecycle methods.
    ///
    /// We have a few assumptions for lifecycle methods:
    /// - They cannot have a return value.
    /// - The export name is prefixed with `canister_`, e.g. `init` => `canister_init`.
    pub fn is_lifecycle(&self) -> bool {
        match self {
            MethodType::Init
            | MethodType::PreUpgrade
            | MethodType::PostUpgrade
            | MethodType::Heartbeat
            | MethodType::InspectMessage
            | MethodType::OnLowWasmMemory => true,
            MethodType::Update | MethodType::Query => false,
        }
    }

    /// `init`, `post_upgrade`, `update`, `query` can have arguments.
    pub fn can_have_args(&self) -> bool {
        match self {
            MethodType::Init | MethodType::PostUpgrade | MethodType::Update | MethodType::Query => {
                true
            }
            MethodType::PreUpgrade
            | MethodType::Heartbeat
            | MethodType::InspectMessage
            | MethodType::OnLowWasmMemory => false,
        }
    }

    pub fn is_state_persistent(&self) -> bool {
        match self {
            Self::Query | Self::InspectMessage => false,
            Self::Update
            | Self::Heartbeat
            | Self::Init
            | Self::PreUpgrade
            | Self::PostUpgrade
            | Self::OnLowWasmMemory => true,
        }
    }
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodType::Init => f.write_str("init"),
            MethodType::PreUpgrade => f.write_str("pre_upgrade"),
            MethodType::PostUpgrade => f.write_str("post_upgrade"),
            MethodType::Query => f.write_str("query"),
            MethodType::Update => f.write_str("update"),
            MethodType::Heartbeat => f.write_str("heartbeat"),
            MethodType::InspectMessage => f.write_str("inspect_message"),
            MethodType::OnLowWasmMemory => f.write_str("on_low_wasm_memory"),
        }
    }
}

fn get_args(method: MethodType, signature: &Signature) -> Result<Vec<Ident>, Error> {
    // Fresh positional bindings avoid collisions with user patterns and paths.
    let mut args = vec![];
    for (i, arg) in signature.inputs.iter().enumerate() {
        let ident = match arg {
            FnArg::Receiver(r) => {
                return Err(Error::new(
                    r.span(),
                    format!("#[{method}] cannot be above functions with `self` as a parameter."),
                ));
            }
            FnArg::Typed(_) => format_ident!("__platform_arg_{i}", span = Span::mixed_site()),
        };

        args.push(ident);
    }

    Ok(args)
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

fn dfn_macro(
    method: MethodType,
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    let attr_span = attr.span();
    let attrs = ExportAttributes::parse(attr)?;
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
    // User-supplied function/codec paths retain their own spans. Generated locals
    // must not capture those paths, even when they have the same spelling.
    let guard_result = Ident::new("r", Span::mixed_site());
    let argument_bytes = Ident::new("arg_bytes", Span::mixed_site());
    let result = Ident::new("result", Span::mixed_site());

    let fun: ItemFn = syn::parse2::<syn::ItemFn>(item.clone()).map_err(|e| {
        Error::new(
            item.span(),
            format!("#[{method}] must be above a function. \n{e}"),
        )
    })?;
    let signature = &fun.sig;
    let generics = &signature.generics;

    if !generics.params.is_empty() {
        return Err(Error::new(
            generics.span(),
            format!("#[{method}] must be above a function with no generic parameters."),
        ));
    }
    let cratename: Path =
        syn::parse_str(attrs.cratename.as_deref().unwrap_or("::ic_platform::cdk"))?;

    // 1. function name(s)
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
    let host_compatible_name = host_export_name(&export_name);

    // 2. guard(s)
    if !attrs.guard.is_empty() && method.is_lifecycle() {
        return Err(Error::new(
            attr_span,
            format!("#[{method}] cannot have guard function(s)."),
        ));
    }
    let guards = attrs
        .guard
        .iter()
        .map(|guard_name| -> Result<_, Error> {
            let guard_path = parse_str::<Path>(guard_name)?;
            Ok(quote! {
                let #guard_result: ::std::result::Result<(), ::std::string::String> = #guard_path ();
                if let ::std::result::Result::Err(e) = #guard_result {
                    #cratename::api::msg_reject(&e);
                    return;
                }
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let guard = quote! {
        #(#guards)*
    };

    // 3. decode arguments
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
    let arg_decode = if let Some(decode_with) = &attrs.decode_with {
        let decode_with_ident = parse_str::<Path>(decode_with)?;
        if arg_tuple.len() == 1 {
            let arg_one = &arg_tuple[0];
            quote! {
                let #argument_bytes = #cratename::api::msg_arg_data();
                let #arg_one = #decode_with_ident(#argument_bytes);
            }
        } else {
            quote! {
            let #argument_bytes = #cratename::api::msg_arg_data();
            let ( #( #arg_tuple, )* ) = #decode_with_ident(#argument_bytes); }
        }
    } else if method.can_have_args() && (!method.is_lifecycle() || !arg_tuple.is_empty()) {
        return Err(Error::new(
            attr_span,
            "endpoint requires decode_with, including unit RPC requests; use the consumer-owned Wire decoder",
        ));
    } else {
        quote! {}
    };

    // 4. function call
    let function_call = if signature.asyncness.is_some() {
        quote! { #name ( #(#arg_tuple),* ) .await }
    } else {
        quote! { #name ( #(#arg_tuple),* ) }
    };

    // 5. return
    let return_length = match &signature.output {
        ReturnType::Default => 0,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.len(),
            _ => 1,
        },
    };
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
    let return_encode = if method.is_lifecycle() || attrs.manual_reply {
        quote! {}
    } else {
        let return_bytes = if let Some(encode_with) = &attrs.encode_with {
            let encode_with_ident = parse_str::<Path>(encode_with)?;
            match return_length {
                0 => quote! { #encode_with_ident()},
                _ => quote! { #encode_with_ident(#result)},
            }
        } else {
            return Err(Error::new(
                attr_span,
                "endpoint requires encode_with or manual_reply; no default Candid codec is supplied",
            ));
        };
        quote! {
            let bytes: ::std::vec::Vec<u8> = #return_bytes;
            #cratename::api::msg_reply(bytes);
        }
    };

    // Codec-defined endpoints are never registered as Candid services.

    // 7. exported function body
    let async_context_name = if method.is_state_persistent() {
        format_ident!("in_executor_context")
    } else {
        format_ident!("in_query_executor_context")
    };
    let body = if signature.asyncness.is_some() {
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

    Ok(quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = #export_name))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = #host_compatible_name))]
        fn #outer_function_ident() {
            #body
        }


        #item
    })
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
