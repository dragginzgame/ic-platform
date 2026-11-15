use super::expand;
use proc_macro2::TokenStream;
use quote::quote;

fn options() -> TokenStream {
    quote!(
        runtime = renamed,
        name = "echo",
        guard = guard,
        decode = decode,
        max_bytes = 8,
        on_error = fail,
        manual_reply
    )
}

#[test]
fn valid_update_generates_handler_and_fixed_export() {
    let output = expand(
        options(),
        quote!(
            async fn echo(input: Vec<u8>) {}
        ),
    )
    .unwrap();
    let file: syn::File = syn::parse2(output).unwrap();
    assert_eq!(file.items.len(), 2);
    let syn::Item::Fn(wrapper) = &file.items[1] else {
        panic!("expected wrapper")
    };
    assert!(wrapper.sig.inputs.is_empty());
    assert!(matches!(wrapper.sig.output, syn::ReturnType::Default));
    assert_eq!(
        wrapper
            .sig
            .abi
            .as_ref()
            .unwrap()
            .name
            .as_ref()
            .unwrap()
            .value(),
        "C"
    );
}

#[test]
fn duplicate_and_unknown_modes_refuse() {
    for extra in [
        quote!(name = "other"),
        quote!(query),
        quote!(encode = encoder),
    ] {
        let args = options();
        assert!(
            expand(
                quote!(#args, #extra),
                quote!(
                    async fn echo(input: Vec<u8>) {}
                )
            )
            .is_err()
        );
    }
}

#[test]
fn all_policy_options_must_be_explicit() {
    assert!(
        expand(
            quote!(runtime = renamed, name = "echo"),
            quote!(
                async fn echo(input: Vec<u8>) {}
            )
        )
        .is_err()
    );
    assert!(
        expand(
            quote!(
                runtime = renamed,
                name = "echo",
                guard = guard,
                decode = decode,
                max_bytes = 8,
                on_error = fail
            ),
            quote!(
                async fn echo(input: Vec<u8>) {}
            )
        )
        .is_err()
    );
}

#[test]
fn empty_and_nul_export_names_refuse() {
    for name in ["", "bad\0name"] {
        assert!(
            expand(
                quote!(runtime = renamed, name = #name, guard = guard, decode = decode,
            max_bytes = 8, on_error = fail, manual_reply),
                quote!(
                    async fn echo(input: Vec<u8>) {}
                )
            )
            .is_err()
        );
    }
}

#[test]
fn unsupported_signatures_refuse() {
    for item in [
        quote!(
            fn echo(input: Vec<u8>) {}
        ),
        quote!(
            async unsafe fn echo(input: Vec<u8>) {}
        ),
        quote!(
            async fn echo<T>(input: T) {}
        ),
        quote!(
            async fn echo() {}
        ),
        quote!(
            async fn echo(a: u8, b: u8) {}
        ),
        quote!(
            async fn echo(input: Vec<u8>) -> u8 {
                0
            }
        ),
    ] {
        assert!(expand(options(), item).is_err());
    }
}

#[test]
fn cfg_is_forwarded_and_attribute_injection_refuses() {
    let output = expand(
        options(),
        quote!(
            #[cfg(any())]
            async fn echo(input: Vec<u8>) {}
        ),
    )
    .unwrap();
    let file: syn::File = syn::parse2(output).unwrap();
    let syn::Item::Fn(wrapper) = &file.items[1] else {
        panic!("expected wrapper")
    };
    assert_eq!(
        wrapper
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg"))
            .count(),
        2
    );
    assert!(
        expand(
            options(),
            quote!(
                #[unsafe(export_name = "other")]
                async fn echo(input: Vec<u8>) {}
            )
        )
        .is_err()
    );
    assert!(
        expand(
            options(),
            quote!(
                #[cfg_attr(all(), unsafe(export_name = "other"))]
                async fn echo(input: Vec<u8>) {}
            )
        )
        .is_err()
    );
}
