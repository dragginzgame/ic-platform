//! Mechanical option parsing shared by the two explicit entrypoint adapters.
use std::collections::BTreeSet;
use syn::{Result, meta::ParseNestedMeta};

pub(crate) fn unique(
    meta: &ParseNestedMeta<'_>,
    name: &str,
    seen: &mut BTreeSet<String>,
    context: &str,
) -> Result<()> {
    if name != "guard" && !seen.insert(name.to_owned()) {
        return Err(meta.error(format!("duplicate {context} option `{name}`")));
    }
    Ok(())
}

pub(crate) fn flag(meta: &ParseNestedMeta<'_>) -> Result<bool> {
    if meta.input.peek(syn::Token![=]) {
        Ok(meta.value()?.parse::<syn::LitBool>()?.value)
    } else {
        Ok(true)
    }
}
