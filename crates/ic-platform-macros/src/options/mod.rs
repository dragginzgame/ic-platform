//! Shared duplicate and boolean admission for entrypoint option parsers.

use std::collections::BTreeSet;
use syn::{LitBool, Token, meta::ParseNestedMeta};

pub(crate) fn unique(
    meta: &ParseNestedMeta<'_>,
    name: &str,
    seen: &mut BTreeSet<String>,
    context: &str,
) -> syn::Result<()> {
    if name == "guard" || seen.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(meta.error(format!("duplicate {context} option `{name}`")))
    }
}

pub(crate) fn flag(meta: &ParseNestedMeta<'_>) -> syn::Result<bool> {
    let name = meta
        .path
        .get_ident()
        .map_or_else(|| "boolean flag".to_owned(), ToString::to_string);
    if meta.input.peek(Token![=]) {
        meta.value()?
            .parse::<LitBool>()
            .map(|literal| literal.value)
            .map_err(|_| {
                meta.error(format!(
                    "option `{name}` requires an actual boolean literal after `=`"
                ))
            })
    } else if meta.input.is_empty() || meta.input.peek(Token![,]) {
        Ok(true)
    } else {
        Err(meta.error(format!(
            "option `{name}` accepts a bare flag or `= true` / `= false`"
        )))
    }
}
