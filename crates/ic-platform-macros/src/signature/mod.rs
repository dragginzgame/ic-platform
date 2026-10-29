//! Shared syntax facts; each adapter retains its own admission and diagnostics.
use syn::{FnArg, ReturnType, Signature, Type};

pub(crate) fn argument_types(
    signature: &Signature,
    receiver_error: impl Fn(&syn::Receiver) -> syn::Error,
) -> syn::Result<Vec<&Type>> {
    signature
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(arg) => Ok(arg.ty.as_ref()),
            FnArg::Receiver(arg) => Err(receiver_error(arg)),
        })
        .collect()
}

pub(crate) enum ReturnShape<'a> {
    Unit,
    Tuple(&'a Type, usize),
    Scalar(&'a Type),
}

impl<'a> ReturnShape<'a> {
    pub(crate) fn of(signature: &'a Signature) -> Self {
        match &signature.output {
            ReturnType::Default => Self::Unit,
            ReturnType::Type(_, ty) => match ty.as_ref() {
                Type::Tuple(tuple) if tuple.elems.is_empty() => Self::Unit,
                Type::Tuple(tuple) => Self::Tuple(ty, tuple.elems.len()),
                _ => Self::Scalar(ty),
            },
        }
    }
    pub(crate) fn arity(&self) -> usize {
        match self {
            Self::Unit => 0,
            Self::Tuple(_, n) => *n,
            Self::Scalar(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
