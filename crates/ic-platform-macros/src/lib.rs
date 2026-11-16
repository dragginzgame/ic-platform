//! Explicit IC exports; codecs and guard policy stay with callers.
#![forbid(unsafe_code)]

mod update;

/// Exports one update handler on wasm32 with an explicit reply mode.
///
/// Required options are `runtime = path`, `name = "method"`, `guard = path`,
/// `decode = path`, `max_bytes = integer`, `on_error = path`, and either
/// `manual_reply` or `encode = path`. Manual handlers are async and return unit;
/// automatic handlers may be sync or async and return the encoder's input. The
/// encoder returns `Result<Vec<u8>, E>`. All handlers take one owned input. The
/// explicit error handler must return the never type (`!`).
/// No implicit codec or retry is selected. Only cfg/cfg_attr gating,
/// documentation and lint attributes are accepted on the handler in this slice.
///
/// Generated exports use a fixed no-argument, no-result C ABI. Their sole unsafe
/// attribute is narrowly expected at that wrapper, requiring a consumer lint
/// policy that permits this IC export boundary. The original handler stays safe.
#[proc_macro_attribute]
pub fn update(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    update::expand(args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Exports a synchronous query with explicit guard, decoder, encoder and bounds.
///
/// Uses the same named options as an automatic update. Async/manual queries are
/// refused; no executor task or callback is created. The error callback returns `!`.
#[proc_macro_attribute]
pub fn query(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    update::expand_kind(args.into(), item.into(), update::Kind::Query)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Exports a synchronous init handler with one explicitly decoded input.
///
/// Requires `runtime`, `guard`, `decode`, `max_bytes`, and `on_error` paths/options.
/// Fixed lifecycle names have no `name`, encoder or reply mode. Handlers return unit.
#[proc_macro_attribute]
pub fn init(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    update::expand_kind(args.into(), item.into(), update::Kind::Init)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Exports an explicit synchronous pre-upgrade handler without reply authority.
///
/// Options match [`init`]. The decoder receives empty bytes, without argument
/// imports. This does not install an automatic upgrade guard or persistence policy.
#[proc_macro_attribute]
pub fn pre_upgrade(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    update::expand_kind(args.into(), item.into(), update::Kind::PreUpgrade)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Exports a synchronous post-upgrade handler with explicit input admission.
///
/// Options match [`init`]. Runtime does not restore heap-local futures or state.
#[proc_macro_attribute]
pub fn post_upgrade(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    update::expand_kind(args.into(), item.into(), update::Kind::PostUpgrade)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
