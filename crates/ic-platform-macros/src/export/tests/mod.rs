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
fn endpoints_require_explicit_codecs() {
    for method in [MethodType::Query, MethodType::Update] {
        let error = dfn_macro(method, quote! {}, quote! { fn endpoint() {} }).unwrap_err();
        assert!(error.to_string().contains("decode_with"));
    }
    for method in [MethodType::Query, MethodType::Update] {
        let error = dfn_macro(
            method,
            quote! { decode_with = "decode" },
            quote! { fn endpoint() {} },
        )
        .unwrap_err();
        assert!(error.to_string().contains("encode_with"));
        let generated = dfn_macro(
            method,
            quote! { decode_with = "decode", manual_reply = true },
            quote! { fn endpoint() {} },
        )
        .unwrap()
        .to_string();
        assert!(!generated.contains("candid"));
        assert!(!generated.contains("api :: msg_reply"));
    }
}

#[test]
fn lifecycle_contexts_and_restrictions() {
    for method in [
        MethodType::Init,
        MethodType::PostUpgrade,
        MethodType::PreUpgrade,
        MethodType::Heartbeat,
        MethodType::InspectMessage,
        MethodType::OnLowWasmMemory,
    ] {
        let attr = if method.can_have_args() {
            quote! { decode_with = "decode" }
        } else {
            quote! {}
        };
        let generated = dfn_macro(method, attr.clone(), quote! { async fn entry() {} })
            .unwrap()
            .to_string();
        assert!(!generated.contains("candid"));
        assert!(generated.contains("futures :: spawn"));
        assert!(!generated.contains("spawn_protected"));
        assert!(
            dfn_macro(
                method,
                quote! { #attr, guard = "guard" },
                quote! { fn entry() {} }
            )
            .is_err()
        );
        assert!(dfn_macro(method, attr, quote! { fn entry() -> bool { true } }).is_err());
        if !method.can_have_args() {
            assert!(
                dfn_macro(
                    method,
                    quote! { decode_with = "decode" },
                    quote! { fn entry() {} }
                )
                .is_err()
            );
        }
    }
}

#[test]
fn names_paths_and_composite_contexts() {
    let generated = ic_query(quote! { crate = "::renamed::cdk", composite = true, name = "named", guard = "first", guard = "second", decode_with = "decode", encode_with = "encode" }, quote! { async fn entry() -> bool { true } }).unwrap().to_string();
    assert!(generated.contains("canister_composite_query named"));
    assert!(
        generated
            .contains(":: renamed :: cdk :: futures :: internals :: in_query_executor_context")
    );
    assert!(generated.find("first ()").unwrap() < generated.find("second ()").unwrap());
    assert!(ic_update(quote! { name = "<ic-cdk internal> reserved", decode_with = "decode", encode_with = "encode" }, quote! { fn entry() {} }).is_err());
    for attr in [
        quote! { crate = "bad path" },
        quote! { decode_with = "value()" },
        quote! { candid = "::candid" },
    ] {
        assert!(ic_update(attr, quote! { fn entry() {} }).is_err());
    }
}

#[test]
fn lifecycle_arguments_are_explicit_and_empty_hooks_stay_raw() {
    for method in [MethodType::Init, MethodType::PostUpgrade] {
        assert!(
            dfn_macro(method, quote! {}, quote! { fn entry(input: Opaque) {} })
                .unwrap_err()
                .to_string()
                .contains("decode_with")
        );
        let generated = dfn_macro(method, quote! {}, quote! { fn entry() {} })
            .unwrap()
            .to_string();
        assert!(!generated.contains("candid"));
        assert!(!generated.contains("msg_arg_data"));
    }
}
