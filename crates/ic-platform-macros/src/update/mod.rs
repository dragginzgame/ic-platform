#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{
    ItemFn, LitInt, LitStr, Meta, Path, ReturnType, Safety, Token, Type, parse::Parser,
    punctuated::Punctuated, spanned::Spanned,
};

#[derive(Default)]
struct Options {
    runtime: Option<Path>,
    name: Option<LitStr>,
    guard: Option<Path>,
    decode: Option<Path>,
    maximum: Option<LitInt>,
    on_error: Option<Path>,
    manual: bool,
    encode: Option<Path>,
}

fn required<T>(value: Option<T>, name: &str, span: proc_macro2::Span) -> syn::Result<T> {
    value.ok_or_else(|| syn::Error::new(span, format!("missing required option `{name}`")))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Update,
    Query,
    Init,
    PreUpgrade,
    PostUpgrade,
}

pub(super) fn expand(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    expand_kind(args, item, Kind::Update)
}

pub(super) fn expand_kind(
    args: TokenStream,
    item: TokenStream,
    kind: Kind,
) -> syn::Result<TokenStream> {
    let mut options = Options::default();
    let mut seen = BTreeSet::new();
    syn::meta::parser(|meta| {
        let Some(key) = meta.path.get_ident() else {
            return Err(meta.error("expected an option name"));
        };
        let key = key.to_string();
        if !seen.insert(key.clone()) {
            return Err(meta.error("duplicate update option"));
        }
        match key.as_str() {
            "runtime" => options.runtime = Some(meta.value()?.parse()?),
            "name" => options.name = Some(meta.value()?.parse()?),
            "guard" => options.guard = Some(meta.value()?.parse()?),
            "decode" => options.decode = Some(meta.value()?.parse()?),
            "max_bytes" => options.maximum = Some(meta.value()?.parse()?),
            "on_error" => options.on_error = Some(meta.value()?.parse()?),
            "manual_reply" => options.manual = true,
            "encode" => options.encode = Some(meta.value()?.parse()?),
            _ => {
                return Err(meta.error("unsupported update option or entrypoint option"));
            }
        }
        Ok(())
    })
    .parse2(args)?;
    let function: ItemFn = syn::parse2(item)?;
    let signature = &function.sig;
    let lifecycle = matches!(kind, Kind::Init | Kind::PreUpgrade | Kind::PostUpgrade);
    if (options.manual && signature.asyncness.is_none())
        || (kind != Kind::Update && signature.asyncness.is_some())
        || signature.constness.is_some()
        || !matches!(signature.safety, Safety::Default)
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.inputs.len() != 1
        || signature.receiver().is_some()
    {
        return Err(syn::Error::new(
            signature.span(),
            "update requires a safe async nongeneric function with one decoded argument",
        ));
    }
    if (options.manual || lifecycle)
        && !matches!(&signature.output, ReturnType::Default)
        && !matches!(&signature.output, ReturnType::Type(_, ty) if matches!(ty.as_ref(), Type::Tuple(tuple) if tuple.elems.is_empty()))
    {
        return Err(syn::Error::new(
            signature.output.span(),
            "manual-reply update must return unit",
        ));
    }
    let mut gating = Vec::new();
    for attribute in &function.attrs {
        let path = attribute.path();
        if path.is_ident("cfg") {
            gating.push(attribute);
        } else if path.is_ident("cfg_attr") {
            let nested =
                attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            if nested.len() < 2
                || nested
                    .iter()
                    .skip(1)
                    .any(|meta| !meta.path().is_ident("cfg"))
            {
                return Err(syn::Error::new(
                    attribute.span(),
                    "cfg_attr may only conditionally apply cfg to an update handler",
                ));
            }
            gating.push(attribute);
        } else if !["doc", "allow", "expect", "warn", "deny", "forbid"]
            .iter()
            .any(|name| path.is_ident(name))
        {
            return Err(syn::Error::new(
                attribute.span(),
                "unsupported attribute on update handler",
            ));
        }
    }
    let span = signature.ident.span();
    let runtime = required(options.runtime, "runtime", span)?;
    let name = if lifecycle {
        if options.name.is_some() {
            return Err(syn::Error::new(span, "lifecycle exports have fixed names"));
        }
        LitStr::new("", span)
    } else {
        required(options.name, "name", span)?
    };
    if !lifecycle && (name.value().is_empty() || name.value().contains('\0')) {
        return Err(syn::Error::new(
            name.span(),
            "update name must be nonempty and contain no NUL",
        ));
    }
    let guard = required(options.guard, "guard", span)?;
    let decode = required(options.decode, "decode", span)?;
    let maximum = required(options.maximum, "max_bytes", span)?;
    let on_error = required(options.on_error, "on_error", span)?;
    if options.manual && (kind != Kind::Update || options.encode.is_some()) {
        return Err(syn::Error::new(
            span,
            "manual_reply is exclusive to updates without an encoder",
        ));
    }
    if lifecycle && options.encode.is_some() {
        return Err(syn::Error::new(
            span,
            "lifecycle exports cannot encode a reply",
        ));
    }
    if !lifecycle && !options.manual && options.encode.is_none() {
        return Err(syn::Error::new(
            span,
            "manual_reply or an explicit encode option is required",
        ));
    }
    let handler = &signature.ident;
    let plain = handler.to_string();
    let prefix = match kind {
        Kind::Update => "update",
        Kind::Query => "query",
        Kind::Init => "init",
        Kind::PreUpgrade => "pre_upgrade",
        Kind::PostUpgrade => "post_upgrade",
    };
    let wrapper = format_ident!(
        "__ic_platform_{}_{}",
        prefix,
        plain.trim_start_matches("r#"),
        span = proc_macro2::Span::mixed_site()
    );
    let export_name = if lifecycle {
        format!("canister_{prefix}")
    } else {
        format!("canister_{prefix} {}", name.value())
    };
    let export = LitStr::new(&export_name, proc_macro2::Span::mixed_site());
    let invocation = if lifecycle {
        let has_arguments = kind != Kind::PreUpgrade;
        quote!(#runtime::entrypoints::lifecycle(#maximum, #has_arguments, #guard, #decode, #handler))
    } else if options.manual {
        quote!(#runtime::entrypoints::update_manual(#maximum, #guard, #decode, #handler))
    } else {
        let encode = options.encode.expect("encoder was required above");
        if kind == Kind::Query {
            quote!(#runtime::entrypoints::query(#maximum, #guard, #decode, #handler, #encode))
        } else {
            let invoke_handler = if signature.asyncness.is_some() {
                quote!(#handler)
            } else {
                quote!(|input| async move { #handler(input) })
            };
            quote!(#runtime::entrypoints::update(#maximum, #guard, #decode, #invoke_handler, #encode, #on_error))
        }
    };
    let gating = &gating;
    // Keep dispatch and ABI wrapper names disjoint for handlers such as
    // observe and observe_dispatch in the same module.
    let dispatch = format_ident!(
        "__ic_platform_dispatch_{}_{}",
        prefix,
        plain.trim_start_matches("r#"),
        span = proc_macro2::Span::mixed_site()
    );
    Ok(quote! {
        #function
        #(#gating)*
        const _: fn() = #dispatch;
        #(#gating)*
        fn #dispatch() {
            match #invocation {
                ::std::result::Result::Ok(_) => {},
                ::std::result::Result::Err(error) => {
                    let failure: fn(_) -> ! = #on_error;
                    failure(error)
                }
            }
        }
        #(#gating)*
        #[cfg(target_arch = "wasm32")]
        #[doc(hidden)]
        #[expect(unsafe_code, reason = "Generated IC entrypoint export with fixed ABI and explicit policies")]
        #[unsafe(export_name = #export)]
        pub extern "C" fn #wrapper() {
            #dispatch();
        }
    })
}
