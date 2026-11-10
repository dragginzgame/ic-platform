//! External codec bridge; the existing entrypoint generator owns all runtime emission.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr, Path, ext::IdentExt, parse::Parser};

pub(crate) fn expand_owned(
    query: bool,
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let mut limits = None;
    let mut reply_bytes = None;
    let mut arity = None;
    let mut empty = None;
    let mut name = None;
    let mut guards = Vec::new();
    let mut composite = false;
    let mut platform: Path = syn::parse_quote!(::ic_platform);
    let mut seen = std::collections::BTreeSet::new();
    syn::meta::parser(|meta| {
        let Some(key) = meta.path.get_ident().map(ToString::to_string) else {
            return Err(meta.error("expected an external Candid option name"));
        };
        crate::options::unique(&meta, &key, &mut seen, "external Candid")?;
        if key == "composite" {
            composite = crate::options::flag(&meta)?;
            return Ok(());
        }
        match key.as_str() {
            "limits" | "empty_args" | "name" | "guard" | "platform" | "reply_bytes" | "arity" => {}
            _ => return Err(meta.error(format!("unsupported external Candid option `{key}`"))),
        }
        let value: LitStr = meta.value()?.parse()?;
        match key.as_str() {
            "limits" => limits = Some(value.parse::<Path>()?),
            "reply_bytes" => reply_bytes = Some(value.parse::<Path>()?),
            "arity" => {
                arity = Some(match value.value().as_str() {
                    "exact" => quote!(Exact),
                    "compatible" => quote!(Compatible),
                    _ => return Err(meta.error("arity must be exact or compatible")),
                })
            }
            "empty_args" => {
                empty = Some(match value.value().as_str() {
                    "reject" => quote!(Reject),
                    "unit" => quote!(Unit),
                    _ => return Err(meta.error("empty_args must be reject or unit")),
                })
            }
            "name" => name = Some(value),
            "guard" => {
                guards.push(value.parse::<Path>()?);
            }
            "platform" => platform = value.parse()?,
            _ => unreachable!(),
        }
        Ok(())
    })
    .parse2(attr)?;
    let limits =
        limits.ok_or_else(|| syn::Error::new_spanned(&item, "external Candid requires limits"))?;
    let empty = empty
        .ok_or_else(|| syn::Error::new_spanned(&item, "external Candid requires empty_args"))?;
    if reply_bytes.is_none() || arity.is_none() {
        return Err(syn::Error::new_spanned(
            &item,
            "owned Candid requires reply_bytes and arity",
        ));
    }
    if composite && !query {
        return Err(syn::Error::new_spanned(
            &item,
            "composite is only supported on query methods",
        ));
    }
    let function: ItemFn = syn::parse2(item.clone())?;
    let ident = function.sig.ident.unraw();
    let decode = format_ident!("__platform_candid_decode_{ident}");
    let encode = format_ident!("__platform_candid_encode_{ident}");
    let admit = format_ident!("__platform_candid_admit_{ident}");
    let types = crate::signature::argument_types(&function.sig, |arg| {
        syn::Error::new_spanned(arg, "self is unsupported on entrypoints")
    })?;
    let tuple = quote!((#(#types,)*));
    let decode_result = if types.len() == 1 {
        quote!(value.0)
    } else {
        quote!(value)
    };
    let decode_type = if types.len() == 1 {
        let ty = types[0];
        quote!(#ty)
    } else {
        tuple.clone()
    };
    let (encode_param, encode_value) = match crate::signature::ReturnShape::of(&function.sig) {
        crate::signature::ReturnShape::Unit => (
            quote!(),
            quote!(#platform::candid_codec::arguments::encode_args(&(), #reply_bytes)),
        ),
        crate::signature::ReturnShape::Tuple(ty, _) => (
            quote!(value: #ty),
            quote!(#platform::candid_codec::arguments::encode_args(&value, #reply_bytes)),
        ),
        crate::signature::ReturnShape::Scalar(ty) => (
            quote!(value: #ty),
            quote!(#platform::candid_codec::encode_one(&value, #reply_bytes)),
        ),
    };
    let method = name.unwrap_or_else(|| LitStr::new(&ident.to_string(), ident.span()));
    guards.push(syn::parse_quote!(#admit));
    let attrs = crate::entrypoint::ExportAttributes {
        name: Some(method.value()),
        guard: guards,
        cratename: Some(syn::parse_quote!(#platform::cdk)),
        decode_with: Some(syn::parse_quote!(#decode)),
        encode_with: Some(syn::parse_quote!(#encode)),
        composite,
        ..Default::default()
    };
    let entry = crate::entrypoint::external_entry(query, attrs, function, item)?;
    let admission = {
        quote! {
            let limits: #platform::candid_codec::decoder::Limits = #limits;
            #platform::candid_codec::endpoint::admit_len(#platform::ic0::msg_arg_data_size(), limits)
                .unwrap_or_else(|error| #platform::cdk::api::trap(::std::format!("{error:?}")));
        }
    };
    let decoding = {
        quote! {
            let value: #tuple = #platform::candid_codec::endpoint::decode(
                &bytes, #limits, #platform::candid_codec::endpoint::EmptyArgs::#empty,
                #platform::candid_codec::decoder::Arity::#arity,
            ).unwrap_or_else(|error| #platform::cdk::api::trap(::std::format!("{error:?}")));
        }
    };
    Ok(quote! {
        fn #admit() -> ::std::result::Result<(), ::std::string::String> {
            #admission
            Ok(())
        }
        fn #decode(bytes: ::std::vec::Vec<u8>) -> #decode_type {
            #decoding
            #decode_result
        }
        fn #encode(#encode_param) -> ::std::vec::Vec<u8> {
            #encode_value.unwrap_or_else(|_| #platform::cdk::api::trap("failed to encode external Candid reply"))
        }
        #entry
    })
}

#[cfg(test)]
mod tests;
