//! Explicit update exports; codecs, guard policy and manual replies stay with callers.
#![forbid(unsafe_code)]

mod update;

/// Exports one async manual-reply update handler on wasm32.
///
/// Required options are `runtime = path`, `name = "method"`, `guard = path`,
/// `decode = path`, `max_bytes = integer`, `on_error = path`, and `manual_reply`.
/// The handler takes exactly one owned decoded input and returns unit. The
/// explicit error handler must return the never type (`!`).
/// No codec, automatic reply or retry is selected. Only cfg/cfg_attr gating,
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
