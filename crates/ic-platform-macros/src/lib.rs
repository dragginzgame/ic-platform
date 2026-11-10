//! Entrypoint attributes with explicit caller-owned codecs.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod entrypoint;
mod external_candid;
mod options;
mod signature;

fn finish(result: syn::Result<TokenStream2>) -> TokenStream {
    let expanded = result.unwrap_or_else(syn::Error::into_compile_error);
    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("{expanded}");
    }
    expanded.into()
}

fn ordinary(kind: entrypoint::Kind, attrs: TokenStream, item: TokenStream) -> TokenStream {
    finish(entrypoint::expand(kind, attrs.into(), item.into()))
}

/// Exports a query with explicit decoding and automatic encoding or manual reply.
#[proc_macro_attribute]
pub fn query(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::Query, attrs, item)
}

/// Exports an update with explicit decoding and automatic encoding or manual reply.
#[proc_macro_attribute]
pub fn update(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::Update, attrs, item)
}

/// Exports initialization; nonempty arguments require an explicit decoder.
#[proc_macro_attribute]
pub fn init(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::Init, attrs, item)
}

/// Exports a unit-returning pre-upgrade hook without arguments or codecs.
#[proc_macro_attribute]
pub fn pre_upgrade(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::PreUpgrade, attrs, item)
}

/// Exports post-upgrade initialization with explicit decoding when needed.
#[proc_macro_attribute]
pub fn post_upgrade(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::PostUpgrade, attrs, item)
}

/// Exports a unit-returning heartbeat hook.
#[proc_macro_attribute]
pub fn heartbeat(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::Heartbeat, attrs, item)
}

/// Exports message inspection in the query executor context.
#[proc_macro_attribute]
pub fn inspect_message(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::InspectMessage, attrs, item)
}

/// Exports a unit-returning low-Wasm-memory hook.
#[proc_macro_attribute]
pub fn on_low_wasm_memory(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ordinary(entrypoint::Kind::OnLowWasmMemory, attrs, item)
}

/// Exports a query through the existing owned external-Candid adapter.
#[proc_macro_attribute]
pub fn codec_query(attrs: TokenStream, item: TokenStream) -> TokenStream {
    finish(external_candid::expand_owned(
        true,
        attrs.into(),
        item.into(),
    ))
}

/// Exports an update through the existing owned external-Candid adapter.
#[proc_macro_attribute]
pub fn codec_update(attrs: TokenStream, item: TokenStream) -> TokenStream {
    finish(external_candid::expand_owned(
        false,
        attrs.into(),
        item.into(),
    ))
}
