use super::*;

#[test]
fn custom_codec_preserves_admission_and_runtime_contexts() {
    for method in [MethodType::Query, MethodType::Update] {
        for function in [
            quote! { fn entry() {} },
            quote! { fn entry(input: Opaque) -> Opaque { input } },
            quote! { async fn entry() -> Opaque { awaited().await } },
            quote! { async fn entry(a: Opaque, b: Opaque) -> (Opaque, Opaque) { (a, b) } },
        ] {
            let generated = dfn_macro(
                method,
                quote! {
                    hidden = true,
                    guard = "codec::admit",
                    decode_with = "codec::decode::<_, { Method::Start as u64 }>",
                    encode_with = "codec::encode"
                },
                function,
            )
            .unwrap()
            .to_string();
            // These statements cross the allocation/effect boundary. In particular,
            // zero Rust arguments must not elide explicit request-frame validation.
            let guard = generated.find("codec :: admit").unwrap();
            let copy = generated.find("api :: msg_arg_data").unwrap();
            let decode = generated.find("codec :: decode").unwrap();
            let handler = generated.find("let result = entry").unwrap();
            let encode = generated.find("codec :: encode").unwrap();
            let reply = generated.find("api :: msg_reply").unwrap();
            assert!(guard < copy && copy < decode && decode < handler);
            assert!(handler < encode && encode < reply);
            assert!(!generated.contains("candid"));
            assert!(!generated.contains(":: ic_cdk ::"));
            assert!(!generated.contains("spawn_protected"));
            let context = if method.is_state_persistent() {
                "in_executor_context"
            } else {
                "in_query_executor_context"
            };
            assert!(generated.contains(&format!("internals :: {context}")));
        }
    }
}

#[test]
fn platform_contexts_and_lifecycle_restrictions() {
    for method in [
        MethodType::Init,
        MethodType::PreUpgrade,
        MethodType::PostUpgrade,
        MethodType::Heartbeat,
        MethodType::InspectMessage,
        MethodType::OnLowWasmMemory,
        MethodType::Query,
        MethodType::Update,
    ] {
        let generated = dfn_macro(method, quote! {}, quote! { async fn entry() {} })
            .unwrap()
            .to_string();
        let context = if method.is_state_persistent() {
            "in_executor_context"
        } else {
            "in_query_executor_context"
        };
        assert!(generated.contains(&format!("internals :: {context}")));
        assert!(generated.contains(":: ic_platform :: cdk :: futures :: spawn"));
        assert!(!generated.contains("spawn_protected"));
        assert!(!generated.contains(":: ic_cdk ::"));
        assert!(
            !generated
                .replace(":: ic_platform :: candid :: contract", ":: facade :: codec")
                .contains(":: candid ::")
        );
        if method.is_lifecycle() {
            assert!(
                dfn_macro(method, quote! { guard = "guard" }, quote! { fn entry() {} }).is_err()
            );
            assert!(dfn_macro(method, quote! {}, quote! { fn entry() -> u64 { 1 } }).is_err());
        }
        if !method.can_have_args() {
            assert!(dfn_macro(method, quote! {}, quote! { fn entry(value: u64) {} }).is_err());
        }
    }
    let typed = ic_update(
        quote! {},
        quote! { async fn entry(value: u64) -> u64 { value } },
    )
    .unwrap()
    .to_string();
    assert!(typed.contains(":: ic_platform :: candid :: contract :: DecoderConfig"));
    assert!(typed.contains("set_skipping_quota (10000)"));
}

#[test]
fn candid_override_preserves_entrypoint_expansion() {
    for (method, function) in [
        (
            MethodType::Query,
            quote! { fn value(input: u64) -> u64 { input } },
        ),
        (
            MethodType::Update,
            quote! { async fn value(input: u64) -> (u64, bool) { (input, true) } },
        ),
        (MethodType::Init, quote! { fn value(input: u64) {} }),
        (MethodType::PostUpgrade, quote! { fn value(input: u64) {} }),
    ] {
        for options in [
            quote! {},
            quote! { hidden = true },
            quote! { decode_with = "decode", encode_with = "encode" },
        ] {
            if method.is_lifecycle() && !options.is_empty() {
                continue;
            }
            let default = dfn_macro(method, options.clone(), function.clone()).unwrap();
            let custom = dfn_macro(
                method,
                quote! { candid = "::facade::codec", #options },
                function.clone(),
            )
            .unwrap();
            assert_eq!(
                custom
                    .to_string()
                    .replace(":: facade :: codec", ":: ic_platform :: candid :: contract"),
                default.to_string(),
            );
        }
    }
    let composite = ic_query(
        quote! { composite = true, candid = "::facade::codec" },
        quote! { async fn value(input: u64) -> u64 { input } },
    )
    .unwrap()
    .to_string();
    assert!(composite.contains(":: facade :: codec :: candid_method (composite_query"));
    assert!(!composite.contains(":: candid ::"));
}

#[test]
fn invalid_candid_path_is_rejected() {
    for path in ["", "not a path", "value()"] {
        assert!(
            ic_query(
                quote! { candid = #path },
                quote! { fn value() -> u64 { 1 } },
            )
            .is_err()
        );
    }
}

#[test]
fn ic_query_empty() {
    let generated = ic_query(
        quote!(),
        quote! {
            fn query() {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    // 0. The exported function
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };

    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(()).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
    // 1. The #[candid_method] over a dummy function
    let expected = quote! {
        #[::ic_platform::candid::contract::candid_method(query, rename = "query")]
        #[allow(unused_variables)]
        fn __candid_method_query() { panic!("candid dummy function called") }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[1] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_return_one_value() {
    let generated = ic_query(
        quote!(),
        quote! {
            fn query() -> u32 {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    // 0. The exported function
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(result).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_return_tuple() {
    let generated = ic_query(
        quote!(),
        quote! {
            fn query() -> (u32, u32) {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };

    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_args(result).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_one_arg() {
    let generated = ic_query(
        quote!(),
        quote! {
            fn query(a: u32) {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let arg_bytes = ::ic_platform::cdk::api::msg_arg_data();
                let arg_bytes: &[u8] = if arg_bytes.is_empty() { b"DIDL\x00\x00" } else { &arg_bytes };
                let mut decoder_config = ::ic_platform::candid::contract::DecoderConfig::new();
                decoder_config.set_skipping_quota(10000);
                let (a,) = ::ic_platform::candid::contract::decode_args_with_config(arg_bytes, &decoder_config).unwrap();
                let result = query(a);
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(()).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_two_args() {
    let generated = ic_query(
        quote!(),
        quote! {
            fn query(a: u32, b: u32) {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let arg_bytes = ::ic_platform::cdk::api::msg_arg_data();
                let arg_bytes: &[u8] = if arg_bytes.is_empty() { b"DIDL\x00\x00" } else { &arg_bytes };
                let mut decoder_config = ::ic_platform::candid::contract::DecoderConfig::new();
                decoder_config.set_skipping_quota(10000);
                let (a, b,) = ::ic_platform::candid::contract::decode_args_with_config(arg_bytes, &decoder_config).unwrap();
                let result = query(a, b);
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(()).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_two_args_return_value() {
    let generated = ic_query(
        quote!(),
        quote! {
            fn query(a: u32, b: u32) -> u64 {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let arg_bytes = ::ic_platform::cdk::api::msg_arg_data();
                let arg_bytes: &[u8] = if arg_bytes.is_empty() { b"DIDL\x00\x00" } else { &arg_bytes };
                let mut decoder_config = ::ic_platform::candid::contract::DecoderConfig::new();
                decoder_config.set_skipping_quota(10000);
                let (a, b,) = ::ic_platform::candid::contract::decode_args_with_config(arg_bytes, &decoder_config).unwrap();
                let result = query(a, b);
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(result).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_export_name() {
    let generated = ic_query(
        quote!(name = "custom_query"),
        quote! {
            fn query() {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query custom_query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.custom_query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(()).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_custom_decoder() {
    let generated = ic_query(
        quote!(decode_with = "custom_decoder"),
        quote! {
            fn query(a: u32) {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    // 0. The exported function
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let arg_bytes = ::ic_platform::cdk::api::msg_arg_data();
                let a = custom_decoder(arg_bytes);
                let result = query(a);
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(()).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
    // 1. The #[candid_method] over a dummy function
    let expected = quote! {
        #[::ic_platform::candid::contract::candid_method(query, rename = "query")]
        #[allow(unused_variables)]
        fn __candid_method_query(arg_bytes: Vec<u8>) { panic!("candid dummy function called") }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[1] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_query_custom_encoder() {
    let generated = ic_query(
        quote!(encode_with = "custom_encoder"),
        quote! {
            fn query() -> u32 {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    // 0. The exported function
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = custom_encoder(result);
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
    // 1. The #[candid_method] over a dummy function
    let expected = quote! {
        #[::ic_platform::candid::contract::candid_method(query, rename = "query")]
        #[allow(unused_variables)]
        fn __candid_method_query() -> Vec<u8> { panic!("candid dummy function called") }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[1] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn ic_guards() {
    let generated = ic_query(
        quote!(guard = "guard1", guard = "guard2"),
        quote! {
            fn query() {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            let r: Result<(), String> = guard1 ();
            if let Err(e) = r {
                ::ic_platform::cdk::api::msg_reject(&e);
                return;
            }
            let r: Result<(), String> = guard2 ();
            if let Err(e) = r {
                ::ic_platform::cdk::api::msg_reject(&e);
                return;
            }
            ::ic_platform::cdk::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(()).unwrap();
                ::ic_platform::cdk::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}

#[test]
fn alternate_crate() {
    let generated = ic_query(
        quote!(crate = "ic_cdk_old"),
        quote! {
            fn query() -> u32 {}
        },
    )
    .unwrap();
    let parsed = syn::parse2::<syn::File>(generated).unwrap();
    assert!(parsed.items.len() == 3);
    // 0. The exported function
    let fn_name = match parsed.items[0] {
        syn::Item::Fn(ref f) => &f.sig.ident,
        _ => panic!("Incorrect parsed AST."),
    };
    let expected = quote! {
        #[cfg_attr(target_family = "wasm", unsafe(export_name = "canister_query query"))]
        #[cfg_attr(not(target_family = "wasm"), unsafe(export_name = "canister_query.query"))]
        fn #fn_name() {
            ic_cdk_old::futures::internals::in_query_executor_context(|| {
                let result = query();
                let bytes: Vec<u8> = ::ic_platform::candid::contract::encode_one(result).unwrap();
                ic_cdk_old::api::msg_reply(bytes);
            });
        }
    };
    let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();
    match &parsed.items[0] {
        syn::Item::Fn(f) => {
            assert_eq!(*f, expected);
        }
        _ => panic!("not a function"),
    };
}
