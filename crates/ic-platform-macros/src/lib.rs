#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::Error;

mod export;
mod external_candid;
mod signature;

fn handle_debug_and_errors<F>(
    cb: F,
    name: &str,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream
where
    F: FnOnce(
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ) -> Result<proc_macro2::TokenStream, Error>,
{
    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("--- IC_CDK_MACROS DEBUG ---");
        eprintln!("{name}\n    attr: {attr}\n    item: {item}");
    }

    let result = cb(attr.into(), item.into());

    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("--------- RESULT  ---------");
        if let Ok(ref stream) = result {
            eprintln!("{stream}");
        }
        eprintln!("---------------------------");
    }

    result.map_or_else(|e| e.to_compile_error().into(), Into::into)
}

/// Generate an explicit-codec query or composite-query entrypoint.
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_query, "ic_query", attr, item)
}

/// Generate an explicit-codec update entrypoint.
#[proc_macro_attribute]
pub fn update(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_update, "ic_update", attr, item)
}

/// Generate an initialization entrypoint with explicit argument decoding.
#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_init, "ic_init", attr, item)
}

/// Generate a pre-upgrade entrypoint in the shared runtime context.
#[proc_macro_attribute]
pub fn pre_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_pre_upgrade, "ic_pre_upgrade", attr, item)
}

/// Generate a post-upgrade entrypoint with explicit argument decoding.
#[proc_macro_attribute]
pub fn post_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_post_upgrade, "ic_post_upgrade", attr, item)
}

/// Generate a heartbeat entrypoint in the shared runtime context.
#[proc_macro_attribute]
pub fn heartbeat(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_heartbeat, "ic_heartbeat", attr, item)
}

/// Generate a message inspection entrypoint in the shared runtime context.
#[proc_macro_attribute]
pub fn inspect_message(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_inspect_message, "ic_inspect_message", attr, item)
}

/// Generate a low-Wasm-memory entrypoint in the shared runtime context.
#[proc_macro_attribute]
pub fn on_low_wasm_memory(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(
        export::ic_on_low_wasm_memory,
        "ic_on_low_wasm_memory",
        attr,
        item,
    )
}

/// Generate a bounded required-external-Candid query with explicit artifact policy.
#[proc_macro_attribute]
pub fn candid_query(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(
        |attr, item| external_candid::expand(true, attr, item),
        "candid_query",
        attr,
        item,
    )
}
/// Generate a bounded required-external-Candid update with explicit artifact policy.
#[proc_macro_attribute]
pub fn candid_update(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(
        |attr, item| external_candid::expand(false, attr, item),
        "candid_update",
        attr,
        item,
    )
}
