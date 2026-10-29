//! External codec bridge; the existing entrypoint generator owns all runtime emission.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr, Path, ext::IdentExt, parse::Parser};

pub(crate) fn expand(
    query: bool,
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let mut limits = None;
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
        if key != "guard" && !seen.insert(key.clone()) {
            return Err(meta.error(format!("duplicate external Candid option `{key}`")));
        }
        if key == "composite" {
            composite = if meta.input.peek(syn::Token![=]) {
                meta.value()?.parse::<syn::LitBool>()?.value
            } else {
                true
            };
            return Ok(());
        }
        match key.as_str() {
            "limits" | "empty_args" | "name" | "guard" | "platform" => {}
            _ => return Err(meta.error(format!("unsupported external Candid option `{key}`"))),
        }
        let value: LitStr = meta.value()?.parse()?;
        match key.as_str() {
            "limits" => limits = Some(value.parse::<Path>()?),
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
            quote!(#platform::candid::contract::encode_args(())),
        ),
        crate::signature::ReturnShape::Tuple(ty, _) => (
            quote!(value: #ty),
            quote!(#platform::candid::contract::encode_args(value)),
        ),
        crate::signature::ReturnShape::Scalar(ty) => (
            quote!(value: #ty),
            quote!(#platform::candid::contract::encode_one(value)),
        ),
    };
    let mode = if composite {
        quote!(composite_query)
    } else if query {
        quote!(query)
    } else {
        quote!(update)
    };
    let method = name.unwrap_or_else(|| LitStr::new(&ident.to_string(), ident.span()));
    guards.push(syn::parse_quote!(#admit));
    let attrs = crate::export::ExportAttributes {
        name: Some(method.value()),
        guard: guards,
        cratename: Some(syn::parse_quote!(#platform::cdk)),
        decode_with: Some(syn::parse_quote!(#decode)),
        encode_with: Some(syn::parse_quote!(#encode)),
        composite,
        ..Default::default()
    };
    let declared = quote! { #[cfg_attr(not(target_family = "wasm"), #platform::candid::endpoint::__candid_method(#mode, rename = #method))] #item };
    let entry = crate::export::external_entry(query, attrs, function, declared)?;
    Ok(quote! {
        fn #admit() -> ::std::result::Result<(), ::std::string::String> {
            let limits: #platform::candid::arguments::ArgumentLimits = #limits;
            limits.admit_len(#platform::ic0::msg_arg_data_size()).unwrap_or_else(|error| #platform::cdk::api::trap(error.to_string()));
            Ok(())
        }
        fn #decode(bytes: ::std::vec::Vec<u8>) -> #decode_type {
            let limits: #platform::candid::arguments::ArgumentLimits = #limits;
            let value: #tuple = limits.decode(&bytes,#platform::candid::arguments::EmptyArgs::#empty).unwrap_or_else(|error| #platform::cdk::api::trap(error.to_string()));
            #decode_result
        }
        fn #encode(#encode_param) -> ::std::vec::Vec<u8> {
            #encode_value.unwrap_or_else(|_| #platform::cdk::api::trap("failed to encode external Candid reply"))
        }
        #entry
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_adapter_reuses_the_runtime_and_preserves_guard_before_copy() {
        let generated = expand(
            true,
            quote!(
                limits = "LIMITS",
                empty_args = "reject",
                guard = "auth",
                platform = "::renamed"
            ),
            quote!(
                fn external(value: u64) -> u64 {
                    value
                }
            ),
        )
        .unwrap()
        .to_string();
        assert!(generated.contains("renamed :: candid :: endpoint :: __candid_method"));
        assert!(generated.contains("renamed :: cdk :: futures :: internals"));
        let auth = generated.find("= auth ()").unwrap();
        let byte_guard = generated[auth..]
            .find("= __platform_candid_admit_external ()")
            .unwrap()
            + auth;
        let copy = generated
            .find("renamed :: cdk :: api :: msg_arg_data")
            .unwrap();
        assert!(auth < byte_guard && byte_guard < copy);
        assert!(!generated.contains(":: candid :: de"));
        assert!(!generated.contains("ic_cdk"));
    }
    #[test]
    fn external_options_require_policy_and_refuse_implicit_codec_overrides() {
        for (attrs, text) in [
            (quote!(), "requires limits"),
            (quote!(limits = "LIMITS"), "requires empty_args"),
            (
                quote!(limits = "LIMITS", empty_args = "guess"),
                "reject or unit",
            ),
            (
                quote!(limits = "A", limits = "B", empty_args = "unit"),
                "duplicate",
            ),
            (
                quote!(
                    limits = "LIMITS",
                    empty_args = "unit",
                    decode_with = "other"
                ),
                "unsupported",
            ),
        ] {
            assert!(
                expand(
                    true,
                    attrs,
                    quote!(
                        fn external() {}
                    )
                )
                .unwrap_err()
                .to_string()
                .contains(text)
            );
        }
    }
}
