#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::Error;

mod export;

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

/// Export the service using the Platform Candid contract or an explicit path.
#[proc_macro]
pub fn export_candid(input: TokenStream) -> TokenStream {
    let candid: syn::Path = if input.is_empty() {
        syn::parse_quote!(::ic_platform::candid::contract)
    } else {
        syn::parse_macro_input!(input as syn::Path)
    };
    quote::quote! {
        #candid::export_service!(#candid);

        /// Return the owned service C string for the Candid extractor.
        #[unsafe(no_mangle)]
        pub fn get_candid_pointer() -> *mut std::os::raw::c_char {
            let c_string = std::ffi::CString::new(__export_service()).unwrap();
            c_string.into_raw()
        }
    }
    .into()
}

/// Generate an upstream-compatible query or composite-query entrypoint.
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_query, "ic_query", attr, item)
}

/// Generate an upstream-compatible update entrypoint.
#[proc_macro_attribute]
pub fn update(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_update, "ic_update", attr, item)
}

/// Generate an upstream-compatible initialization entrypoint.
#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_init, "ic_init", attr, item)
}

/// Generate an upstream-compatible pre-upgrade entrypoint.
#[proc_macro_attribute]
pub fn pre_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_pre_upgrade, "ic_pre_upgrade", attr, item)
}

/// Generate an upstream-compatible post-upgrade entrypoint.
#[proc_macro_attribute]
pub fn post_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_post_upgrade, "ic_post_upgrade", attr, item)
}

/// Generate an upstream-compatible heartbeat entrypoint.
#[proc_macro_attribute]
pub fn heartbeat(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_heartbeat, "ic_heartbeat", attr, item)
}

/// Generate an upstream-compatible message inspection entrypoint.
#[proc_macro_attribute]
pub fn inspect_message(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_inspect_message, "ic_inspect_message", attr, item)
}

/// Generate an upstream-compatible low-Wasm-memory entrypoint.
#[proc_macro_attribute]
pub fn on_low_wasm_memory(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(
        export::ic_on_low_wasm_memory,
        "ic_on_low_wasm_memory",
        attr,
        item,
    )
}
