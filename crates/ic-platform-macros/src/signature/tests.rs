use super::*;
use quote::{ToTokens, quote};
use std::cell::Cell;

#[test]
fn return_shapes_are_syntactic_including_parentheses_aliases_and_impl_trait() {
    for (tokens, expected_arity, unit) in [
        (
            quote!(
                fn endpoint() {}
            ),
            0,
            true,
        ),
        (
            quote!(
                fn endpoint() -> () {}
            ),
            0,
            true,
        ),
        (
            quote!(
                fn endpoint() -> (u8,) {
                    (1,)
                }
            ),
            1,
            false,
        ),
        (
            quote!(
                fn endpoint() -> (u8, bool) {
                    (1, true)
                }
            ),
            2,
            false,
        ),
        (
            quote!(
                fn endpoint() -> UnitAlias {
                    ()
                }
            ),
            1,
            false,
        ),
        (
            quote!(
                fn endpoint() -> (()) {
                    ()
                }
            ),
            1,
            false,
        ),
        (
            quote!(
                fn endpoint() -> impl ::std::fmt::Debug {
                    1
                }
            ),
            1,
            false,
        ),
    ] {
        let function: syn::ItemFn = syn::parse2(tokens).unwrap();
        let shape = ReturnShape::of(&function.sig);
        assert_eq!(shape.arity(), expected_arity);
        assert_eq!(matches!(shape, ReturnShape::Unit), unit);
        if let ReturnShape::Tuple(ty, _) | ReturnShape::Scalar(ty) = shape {
            let ReturnType::Type(_, original) = &function.sig.output else {
                panic!("expected return type")
            };
            assert!(std::ptr::eq(ty, original.as_ref()));
        }
    }
}

#[test]
fn argument_types_preserve_position_and_ignore_pattern_spelling() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn endpoint((left, right): (u8, bool), mut count: u64, _: String, [a, b]: [u16; 2]) {}
    };
    let types = argument_types(&function.sig, |_| panic!("no receiver in this fixture")).unwrap();
    let rendered = types
        .iter()
        .map(|ty| ty.to_token_stream().to_string())
        .collect::<Vec<_>>();
    assert_eq!(rendered, ["(u8 , bool)", "u64", "String", "[u16 ; 2]"]);
}

#[test]
fn receiver_refusal_uses_the_supplied_error_factory() {
    let function: syn::ItemFn = syn::parse_quote!(
        fn endpoint(&self, value: u64) {}
    );
    let calls = Cell::new(0);
    let result = argument_types(&function.sig, |receiver| {
        calls.set(calls.get() + 1);
        syn::Error::new_spanned(receiver, "custom receiver refusal")
    });
    let error = match result {
        Ok(_) => panic!("receiver must be refused"),
        Err(error) => error,
    };
    assert_eq!(error.to_string(), "custom receiver refusal");
    assert_eq!(calls.get(), 1);
}
