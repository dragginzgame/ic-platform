//! One entrypoint generator for explicit-codec ordinary and typed adapters.

use crate::{
    options,
    signature::{ReturnShape, argument_types},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::BTreeSet;
use syn::{
    Attribute, Error, ItemFn, LitStr, Meta, Path, Token, parse::Parser, punctuated::Punctuated,
    spanned::Spanned,
};

pub(crate) struct ExportAttributes<P = String> {
    pub name: Option<String>,
    pub guard: Vec<P>,
    pub decode_with: Option<P>,
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

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Query,
    Update,
    Init,
    PreUpgrade,
    PostUpgrade,
    Heartbeat,
    InspectMessage,
    OnLowWasmMemory,
}

impl Kind {
    fn rpc(self) -> bool {
        matches!(self, Self::Query | Self::Update)
    }

    fn permits_arguments(self) -> bool {
        matches!(
            self,
            Self::Query | Self::Update | Self::Init | Self::PostUpgrade
        )
    }

    fn query_context(self) -> bool {
        matches!(self, Self::Query | Self::InspectMessage)
    }

    fn export_name(self, name: &str, composite: bool) -> String {
        match self {
            Self::Query if composite => format!("canister_composite_query {name}"),
            Self::Query => format!("canister_query {name}"),
            Self::Update => format!("canister_update {name}"),
            Self::Init => "canister_init".to_owned(),
            Self::PreUpgrade => "canister_pre_upgrade".to_owned(),
            Self::PostUpgrade => "canister_post_upgrade".to_owned(),
            Self::Heartbeat => "canister_heartbeat".to_owned(),
            Self::InspectMessage => "canister_inspect_message".to_owned(),
            Self::OnLowWasmMemory => "canister_on_low_wasm_memory".to_owned(),
        }
    }
}

trait ResolvePath {
    fn resolve(self) -> syn::Result<Path>;
}

impl ResolvePath for LitStr {
    fn resolve(self) -> syn::Result<Path> {
        self.parse()
    }
}

impl ResolvePath for Path {
    fn resolve(self) -> syn::Result<Path> {
        Ok(self)
    }
}

fn parse_options(tokens: TokenStream) -> syn::Result<ExportAttributes<LitStr>> {
    let mut attrs = ExportAttributes::default();
    let mut seen = BTreeSet::new();
    syn::meta::parser(|meta| {
        let Some(identifier) = meta.path.get_ident() else {
            return Err(meta.error("entrypoint option names must be single identifiers"));
        };
        let name = identifier.to_string();
        if name == "hidden" {
            return Err(meta.error("the `hidden` entrypoint option is retired"));
        }
        options::unique(&meta, &name, &mut seen, "entrypoint")?;
        match name.as_str() {
            "name" => attrs.name = Some(meta.value()?.parse::<LitStr>()?.value()),
            "guard" => attrs.guard.push(meta.value()?.parse::<LitStr>()?),
            "decode_with" => attrs.decode_with = Some(meta.value()?.parse::<LitStr>()?),
            "encode_with" => attrs.encode_with = Some(meta.value()?.parse::<LitStr>()?),
            "crate" => attrs.cratename = Some(meta.value()?.parse::<LitStr>()?),
            "manual_reply" => attrs.manual_reply = options::flag(&meta)?,
            "composite" => attrs.composite = options::flag(&meta)?,
            _ => return Err(meta.error(format!("unknown entrypoint option `{name}`"))),
        }
        Ok(())
    })
    .parse2(tokens)?;
    Ok(attrs)
}

fn check_mode<P>(kind: Kind, attrs: &ExportAttributes<P>) -> syn::Result<()> {
    if attrs.composite && !matches!(kind, Kind::Query) {
        return Err(Error::new(
            Span::call_site(),
            "`composite = true` is allowed only on queries",
        ));
    }
    if attrs.manual_reply && !kind.rpc() {
        return Err(Error::new(
            Span::call_site(),
            "lifecycle hooks do not accept `manual_reply = true`",
        ));
    }
    Ok(())
}

pub(crate) fn expand(
    kind: Kind,
    attrs: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let attrs = parse_options(attrs)?;
    check_mode(kind, &attrs)?;
    let function = syn::parse2::<ItemFn>(item.clone())?;
    generate(kind, attrs, function, item)
}

pub(crate) fn external_entry(
    query: bool,
    attrs: ExportAttributes<Path>,
    function: ItemFn,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let kind = if query { Kind::Query } else { Kind::Update };
    check_mode(kind, &attrs)?;
    generate(kind, attrs, function, item)
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Expansion consumes the parsed function and original tokens as one syntax transaction"
)]
fn generate<P: ResolvePath>(
    kind: Kind,
    attrs: ExportAttributes<P>,
    function: ItemFn,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    if !function.sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &function.sig.generics,
            "entrypoint functions cannot have generic parameters",
        ));
    }
    let runtime = match attrs.cratename {
        Some(path) => path.resolve()?,
        None => syn::parse_quote!(::ic_platform_runtime::cdk),
    };
    let handler = &function.sig.ident;
    let spelling = handler.to_string();
    let rust_name = spelling.strip_prefix("r#").unwrap_or(&spelling);
    if !kind.rpc() && attrs.name.is_some() {
        return Err(Error::new(
            handler.span(),
            "lifecycle hooks do not accept a custom `name`",
        ));
    }
    let name = attrs.name.as_deref().unwrap_or(rust_name);
    if name.starts_with("<ic-cdk internal>") {
        return Err(Error::new(
            handler.span(),
            "entrypoint `name` uses the reserved `<ic-cdk internal>` prefix",
        ));
    }
    if !kind.rpc() && !attrs.guard.is_empty() {
        return Err(Error::new(
            handler.span(),
            "lifecycle hooks do not accept `guard`",
        ));
    }
    let guards: Vec<Path> = attrs
        .guard
        .into_iter()
        .map(ResolvePath::resolve)
        .collect::<syn::Result<_>>()?;
    let types = argument_types(&function.sig, |receiver| {
        Error::new_spanned(receiver, "entrypoint functions cannot have a receiver")
    })?;
    if !kind.permits_arguments() && !types.is_empty() {
        return Err(Error::new_spanned(
            &function.sig.inputs,
            "this lifecycle hook cannot take arguments",
        ));
    }
    if !kind.permits_arguments() && attrs.decode_with.is_some() {
        return Err(Error::new(
            handler.span(),
            "this lifecycle hook does not accept `decode_with`",
        ));
    }
    let decoder = match attrs.decode_with {
        Some(path) => Some(path.resolve()?),
        None if kind.rpc() || !types.is_empty() => {
            return Err(Error::new(
                handler.span(),
                "this entrypoint requires explicit `decode_with`",
            ));
        }
        None => None,
    };
    let shape = ReturnShape::of(&function.sig);
    if !kind.rpc() && shape.arity() != 0 {
        return Err(Error::new_spanned(
            &function.sig.output,
            "lifecycle hooks must return syntactic unit",
        ));
    }
    if !kind.rpc() && attrs.encode_with.is_some() {
        return Err(Error::new(
            handler.span(),
            "lifecycle hooks do not accept `encode_with`",
        ));
    }
    // Manual reply deliberately leaves any unused encoder string unparsed.
    let encoder = if kind.rpc() && !attrs.manual_reply {
        Some(
            attrs
                .encode_with
                .ok_or_else(|| {
                    Error::new(
                        handler.span(),
                        "automatic replies require explicit `encode_with`",
                    )
                })?
                .resolve()?,
        )
    } else {
        None
    };

    let bindings: Vec<Ident> = (0..types.len())
        .map(|index| Ident::new(&format!("__ic_argument_{index}"), Span::mixed_site()))
        .collect();
    let bytes = Ident::new("__ic_argument_bytes", Span::mixed_site());
    let decoded = if let Some(decoder) = decoder {
        let assignment = match types.as_slice() {
            [] => quote!(let (): () = #decoder(#bytes);),
            [_] => {
                let binding = &bindings[0];
                quote!(let #binding = #decoder(#bytes);)
            }
            _ => quote!(let (#(#bindings,)*) = #decoder(#bytes);),
        };
        quote! {
            let #bytes = #runtime::api::msg_arg_data();
            #assignment
        }
    } else {
        TokenStream::new()
    };
    let invocation = if function.sig.asyncness.is_some() {
        quote!(#handler(#(#bindings),*).await)
    } else {
        quote!(#handler(#(#bindings),*))
    };
    let body = if let Some(encoder) = encoder {
        let result = Ident::new("__ic_result", Span::mixed_site());
        let reply = Ident::new("__ic_reply_bytes", Span::mixed_site());
        let encode = match shape {
            ReturnShape::Unit => quote! {
                #invocation;
                let #reply: ::std::vec::Vec<::std::primitive::u8> = #encoder();
            },
            ReturnShape::Tuple(_, _) | ReturnShape::Scalar(_) => quote! {
                let #result = #invocation;
                let #reply: ::std::vec::Vec<::std::primitive::u8> = #encoder(#result);
            },
        };
        quote! { #decoded #encode #runtime::api::msg_reply(#reply); }
    } else {
        let ignored_result = Ident::new("_result", Span::mixed_site());
        quote! { #decoded let #ignored_result = #invocation; }
    };
    let guard_result = Ident::new("__ic_guard_result", Span::mixed_site());
    let guard_error = Ident::new("__ic_guard_error", Span::mixed_site());
    let guard_checks = guards
        .iter()
        .map(|guard| {
            quote! {
                let #guard_result: ::std::result::Result<(), ::std::string::String> = #guard();
                if let ::std::result::Result::Err(#guard_error) = #guard_result {
                    #runtime::api::msg_reject(#guard_error.as_str());
                    return;
                }
            }
        })
        .collect::<TokenStream>();
    let context = if kind.query_context() {
        quote!(#runtime::futures::internals::in_query_executor_context)
    } else {
        quote!(#runtime::futures::internals::in_executor_context)
    };
    let execution = if function.sig.asyncness.is_some() {
        quote! {
            #context(|| {
                #guard_checks
                #[allow(clippy::disallowed_methods, reason = "The generator selects the existing protected spawn; consumers may forbid this method or leave the lint disabled")]
                #runtime::futures::spawn(async move { #body });
            });
        }
    } else {
        quote! { #guard_checks #context(|| { #body }); }
    };
    let export_name = kind.export_name(name, attrs.composite);
    let native_name = native_symbol(&export_name);
    let wrapper = Ident::new(
        &format!("__canister_method_{rust_name}"),
        Span::mixed_site(),
    );
    let conditional_attrs = conditional_attributes(&function.attrs)?;
    Ok(quote! {
        #item
        #(#conditional_attrs)*
        #[cfg_attr(target_family = "wasm", unsafe(export_name = #export_name))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = ::std::concat!(::std::env!("CARGO_CRATE_NAME"), "__", #native_name)))]
        fn #wrapper() { #execution }
    })
}

fn native_symbol(export_name: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut symbol = String::from("__ic_export_");
    for byte in export_name.bytes() {
        symbol.push(char::from(HEX[usize::from(byte >> 4)]));
        symbol.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    symbol
}

fn conditional_attributes(attributes: &[Attribute]) -> syn::Result<Vec<TokenStream>> {
    attributes
        .iter()
        .filter_map(|attribute| match conditional_meta(&attribute.meta) {
            Ok(Some(meta)) => Some(Ok(quote!(#[#meta]))),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn conditional_meta(meta: &Meta) -> syn::Result<Option<Meta>> {
    if meta.path().is_ident("cfg") {
        return Ok(Some(meta.clone()));
    }
    if !meta.path().is_ident("cfg_attr") {
        return Ok(None);
    }
    let Meta::List(list) = meta else {
        return Err(Error::new_spanned(
            meta,
            "expected a `cfg_attr` predicate and attributes",
        ));
    };
    let mut arguments = list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
        .into_iter();
    let predicate = arguments
        .next()
        .ok_or_else(|| Error::new_spanned(meta, "missing `cfg_attr` predicate"))?;
    let mut retained = Vec::new();
    for argument in arguments {
        if let Some(attribute) = conditional_meta(&argument)? {
            retained.push(attribute);
        }
    }
    if retained.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            syn::parse_quote_spanned!(meta.span()=> cfg_attr(#predicate, #(#retained),*)),
        ))
    }
}

#[cfg(test)]
mod tests;
