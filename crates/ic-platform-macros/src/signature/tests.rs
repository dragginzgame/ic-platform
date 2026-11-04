use super::*;
#[test]
fn syntactic_shapes_keep_explicit_tuple_and_scalar_arity() {
    for (signature, arity) in [
        (syn::parse_quote!(fn f()), 0),
        (syn::parse_quote!(fn f() -> ()), 0),
        (syn::parse_quote!(fn f() -> (u64,)), 1),
        (syn::parse_quote!(fn f() -> (u64,u32)), 2),
        (syn::parse_quote!(fn f() -> Result<(), E>), 1),
    ] {
        assert_eq!(ReturnShape::of(&signature).arity(), arity);
    }
    let signature = syn::parse_quote!(fn f((a,b): (u64,u64), mut c: u32));
    assert_eq!(
        argument_types(&signature, |arg| syn::Error::new_spanned(arg, "receiver"))
            .unwrap()
            .len(),
        2
    );
}
