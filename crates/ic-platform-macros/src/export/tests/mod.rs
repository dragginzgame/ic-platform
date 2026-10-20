use super::*;

#[test]
fn raw_rust_identifiers_do_not_leak_into_default_wire_names() {
    let generated = ic_query(
        quote! { decode_with = "decode", encode_with = "encode" },
        quote! { fn r#match(value: u64) -> u64 { value } },
    )
    .unwrap()
    .to_string();
    assert!(generated.contains("\"canister_query match\""));
    assert!(!generated.contains("canister_query r#match"));
    let renamed = ic_query(
        quote! { name = "r#explicit", decode_with = "decode", encode_with = "encode" },
        quote! { fn r#match(value: u64) -> u64 { value } },
    )
    .unwrap()
    .to_string();
    assert!(renamed.contains("\"canister_query r#explicit\""));
}

#[test]
fn native_symbols_preserve_distinct_export_names() {
    let names = [
        "canister_query a-b",
        "canister_query a_b",
        "canister_query a b",
        "canister_query a.b",
        "canister_query a<b",
        "canister_query a>b",
        "canister_query café",
        "canister_query cafe",
        "canister_update a-b",
    ];
    let symbols: std::collections::BTreeSet<_> =
        names.iter().map(|name| host_export_name(name)).collect();
    assert_eq!(symbols.len(), names.len());
    assert!(symbols.iter().all(|symbol| {
        symbol.starts_with("__ic_export_")
            && symbol
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    }));
    let generated = ic_query(
        quote! { name = "a-b", decode_with = "decode", encode_with = "encode" },
        quote! { fn entry(value: u64) -> u64 { value } },
    )
    .unwrap()
    .to_string();
    assert!(generated.contains("canister_query a-b"));
    assert!(generated.contains(&host_export_name("canister_query a-b")));
}

#[test]
fn options_preserve_flags_and_reject_ambiguous_or_retired_syntax() {
    for tokens in [
        quote! { manual_reply, composite, guard = "a", guard = "b", },
        quote! { manual_reply = true, composite = true, guard = "a", guard = "b" },
    ] {
        let attrs = ExportAttributes::parse(tokens).unwrap();
        assert!(attrs.manual_reply && attrs.composite);
        assert_eq!(attrs.guard, ["a", "b"]);
    }
    let attrs = ExportAttributes::parse(quote! {
        manual_reply = false, composite = false, crate = "::renamed::cdk"
    })
    .unwrap();
    assert!(!attrs.manual_reply && !attrs.composite);
    assert_eq!(attrs.cratename.as_deref(), Some("::renamed::cdk"));
    for (tokens, diagnostic) in [
        (quote! { hidden = true }, "hidden is retired"),
        (quote! { hidden = false }, "hidden is retired"),
        (quote! { name = "a", name = "b" }, "duplicate"),
        (quote! { manual_reply, manual_reply = false }, "duplicate"),
        (quote! { decode_with = "a", decode_with = "b" }, "duplicate"),
        (quote! { unknown = true }, "unknown"),
        (quote! { guard = 7 }, "string literal"),
        (quote! { composite = "true" }, "boolean literal"),
    ] {
        let error = ExportAttributes::parse(tokens).err().expect("must reject");
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
}

#[test]
fn argument_patterns_need_only_binding_names() {
    let generated = ic_update(
        quote! { decode_with = "decode", encode_with = "encode" },
        quote! { fn entry(entry: Vec<Option<u64>>, (left, right): (u64, u64)) {} },
    )
    .unwrap()
    .to_string();
    assert!(generated.contains("__platform_arg_0"));
    assert!(generated.contains("__platform_arg_1"));
}

#[test]
fn misplaced_execution_options_are_not_silently_ignored() {
    for method in [
        MethodType::Update,
        MethodType::Init,
        MethodType::PreUpgrade,
        MethodType::PostUpgrade,
        MethodType::Heartbeat,
        MethodType::InspectMessage,
        MethodType::OnLowWasmMemory,
    ] {
        let error = dfn_macro(
            method,
            quote! { composite = true },
            quote! { fn entry() {} },
        )
        .unwrap_err();
        assert!(error.to_string().contains("only supported on query"));
        if method.is_lifecycle() {
            let error =
                dfn_macro(method, quote! { manual_reply }, quote! { fn entry() {} }).unwrap_err();
            assert!(error.to_string().contains("not supported on lifecycle"));
        }
    }
}

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
