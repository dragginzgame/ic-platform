use super::*;
use quote::ToTokens;
use syn::{Block, Expr, ExprCall, Item, Pat, Stmt};

fn functions(kind: Kind, attrs: TokenStream, item: TokenStream) -> (ItemFn, ItemFn) {
    let file: syn::File = syn::parse2(expand(kind, attrs, item).unwrap()).unwrap();
    let mut functions: Vec<ItemFn> = file
        .items
        .into_iter()
        .map(|item| match item {
            Item::Fn(function) => function,
            _ => panic!("expected only the original function and wrapper"),
        })
        .collect();
    assert_eq!(functions.len(), 2);
    let wrapper = functions.pop().unwrap();
    (functions.pop().unwrap(), wrapper)
}

fn call(expression: &Expr) -> &ExprCall {
    match expression {
        Expr::Call(call) => call,
        _ => panic!("expected a function call"),
    }
}

fn last_call(block: &Block) -> &ExprCall {
    match block.stmts.last().unwrap() {
        Stmt::Expr(expression, _) => call(expression),
        _ => panic!("expected a final function call"),
    }
}

fn called_name(call: &ExprCall) -> String {
    match call.func.as_ref() {
        Expr::Path(path) => path.path.segments.last().unwrap().ident.to_string(),
        _ => panic!("expected a path call"),
    }
}

fn context_body(wrapper: &ItemFn) -> &Block {
    let context = last_call(&wrapper.block);
    match context.args.first().unwrap() {
        Expr::Closure(closure) => match closure.body.as_ref() {
            Expr::Block(block) => &block.block,
            _ => panic!("expected a context block"),
        },
        _ => panic!("expected a context closure"),
    }
}

fn local_call(block: &Block, index: usize) -> &ExprCall {
    match &block.stmts[index] {
        Stmt::Local(local) => call(&local.init.as_ref().unwrap().expr),
        _ => panic!("expected a local initialized by a call"),
    }
}

fn wasm_name(wrapper: &ItemFn) -> String {
    let args = wrapper.attrs[0]
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .unwrap();
    let Meta::List(unsafe_meta) = &args[1] else {
        panic!("expected unsafe export attribute")
    };
    let Meta::NameValue(export) = unsafe_meta.parse_args::<Meta>().unwrap() else {
        panic!("expected export name")
    };
    let Expr::Lit(literal) = export.value else {
        panic!("expected a literal Wasm export")
    };
    let syn::Lit::Str(name) = literal.lit else {
        panic!("expected an export string")
    };
    name.value()
}

#[test]
fn parser_keeps_guard_order_paths_flags_and_a_trailing_comma() {
    let attrs = parse_options(quote! {
        name = "", guard = "::guards::one", guard = "second::<u8>",
        decode_with = "::codec::decode::<u32>", encode_with = "encoder",
        crate = "crate::runtime", manual_reply = false, composite,
    })
    .unwrap();
    assert_eq!(attrs.name.as_deref(), Some(""));
    assert_eq!(
        attrs.guard.iter().map(LitStr::value).collect::<Vec<_>>(),
        ["::guards::one", "second::<u8>"]
    );
    let path = attrs.decode_with.unwrap().resolve().unwrap();
    assert!(path.leading_colon.is_some());
    assert!(matches!(
        path.segments.last().unwrap().arguments,
        syn::PathArguments::AngleBracketed(_)
    ));
    assert!(!attrs.manual_reply);
    assert!(attrs.composite);

    struct NoDefault;
    let default: ExportAttributes<NoDefault> = ExportAttributes::default();
    assert!(default.name.is_none() && default.guard.is_empty());
    assert!(!default.manual_reply && !default.composite);
}

#[test]
fn parser_refuses_duplicate_retired_unknown_and_wrong_value_forms() {
    for attrs in [
        quote!(name = "a", name = "b"),
        quote!(decode_with = "a", decode_with = "b"),
        quote!(manual_reply, manual_reply = false),
        quote!(composite = "true"),
        quote!(manual_reply(yes)),
        quote!(guard("guard")),
        quote!(nested::guard = "guard"),
        quote!(name = 7),
        quote!(decode_with = decoder),
        quote!(unknown = "x"),
        quote!(hidden = false),
    ] {
        assert!(parse_options(attrs).is_err());
    }
    let duplicate = expand(
        Kind::Query,
        quote!(name = "a", name = "b"),
        quote!(
            struct Wrong;
        ),
    )
    .unwrap_err();
    assert!(duplicate.to_string().contains("duplicate"));
}

#[test]
fn every_kind_has_the_exact_export_and_consumer_scoped_native_name() {
    for (kind, expected, query_context) in [
        (Kind::Query, "canister_query endpoint", true),
        (Kind::Update, "canister_update endpoint", false),
        (Kind::Init, "canister_init", false),
        (Kind::PreUpgrade, "canister_pre_upgrade", false),
        (Kind::PostUpgrade, "canister_post_upgrade", false),
        (Kind::Heartbeat, "canister_heartbeat", false),
        (Kind::InspectMessage, "canister_inspect_message", true),
        (Kind::OnLowWasmMemory, "canister_on_low_wasm_memory", false),
    ] {
        let attrs = if kind.rpc() {
            quote!(decode_with = "decode", encode_with = "encode")
        } else {
            quote!()
        };
        let (_, wrapper) = functions(
            kind,
            attrs,
            quote!(
                fn endpoint() {}
            ),
        );
        assert_eq!(wasm_name(&wrapper), expected);
        assert_eq!(wrapper.sig.ident, "__canister_method_endpoint");
        assert!(wrapper.sig.abi.is_none());
        let native = wrapper.attrs[1].to_token_stream().to_string();
        assert!(native.contains("CARGO_CRATE_NAME"));
        assert!(native.contains(&native_symbol(expected)));
        let context = called_name(last_call(&wrapper.block));
        assert_eq!(
            context,
            if query_context {
                "in_query_executor_context"
            } else {
                "in_executor_context"
            }
        );
    }
    assert_eq!(native_symbol("é?"), "__ic_export_c3a93f");
    assert_ne!(native_symbol("a-b"), native_symbol("a_b"));
}

#[test]
fn raw_custom_empty_and_reserved_names_follow_the_wire_contract() {
    let attrs = quote!(decode_with = "decode", encode_with = "encode");
    let (_, wrapper) = functions(
        Kind::Query,
        attrs,
        quote!(
            fn r#type() {}
        ),
    );
    assert_eq!(wasm_name(&wrapper), "canister_query type");
    assert_eq!(wrapper.sig.ident, "__canister_method_type");
    for name in ["", "é!?-."] {
        let (_, wrapper) = functions(
            Kind::Query,
            quote!(decode_with = "decode", encode_with = "encode", name = #name, composite),
            quote!(
                fn endpoint() {}
            ),
        );
        assert_eq!(
            wasm_name(&wrapper),
            format!("canister_composite_query {name}")
        );
        assert_eq!(wrapper.sig.ident, "__canister_method_endpoint");
    }
    let error = expand(
        Kind::Update,
        quote!(
            decode_with = "decode",
            encode_with = "encode",
            name = "<ic-cdk internal>reserved"
        ),
        quote!(
            fn endpoint() {}
        ),
    )
    .unwrap_err();
    assert!(error.to_string().contains("reserved"));
}

#[test]
fn decoder_patterns_are_positional_and_adapt_only_argument_count() {
    for (function, count) in [
        (
            quote!(
                fn endpoint() {}
            ),
            0,
        ),
        (
            quote!(
                fn endpoint((left, right): (u8, u8)) {}
            ),
            1,
        ),
        (
            quote!(
                fn endpoint(mut value: u8, _: bool, [left, right]: [u16; 2]) {}
            ),
            3,
        ),
    ] {
        let expected_original = function.to_string();
        let (original, wrapper) = functions(
            Kind::Update,
            quote!(decode_with = "decode", encode_with = "encode"),
            function,
        );
        assert_eq!(original.to_token_stream().to_string(), expected_original);
        let body = context_body(&wrapper);
        assert_eq!(called_name(local_call(body, 0)), "msg_arg_data");
        assert_eq!(called_name(local_call(body, 1)), "decode");
        let Stmt::Local(binding) = &body.stmts[1] else {
            panic!("expected decoder binding")
        };
        match (&binding.pat, count) {
            (Pat::Type(typed), 0) => {
                assert!(matches!(typed.pat.as_ref(), Pat::Tuple(tuple) if tuple.elems.is_empty()))
            }
            (Pat::Ident(_), 1) => {}
            (Pat::Tuple(tuple), count) => assert_eq!(tuple.elems.len(), count),
            _ => panic!("wrong positional decoder binding"),
        }
    }
}

#[test]
fn return_encoding_uses_whole_results_and_an_explicit_byte_vector() {
    for (function, arity) in [
        (
            quote!(
                fn endpoint() {}
            ),
            0,
        ),
        (
            quote!(
                fn endpoint() -> () {}
            ),
            0,
        ),
        (
            quote!(
                fn endpoint() -> u64 {
                    1
                }
            ),
            1,
        ),
        (
            quote!(
                fn endpoint() -> (u8, bool) {
                    (1, true)
                }
            ),
            1,
        ),
        (
            quote!(
                fn endpoint() -> (()) {
                    ()
                }
            ),
            1,
        ),
        (
            quote!(
                fn endpoint() -> impl ::std::fmt::Debug {
                    1
                }
            ),
            1,
        ),
    ] {
        let (_, wrapper) = functions(
            Kind::Update,
            quote!(decode_with = "decode", encode_with = "encode"),
            function,
        );
        let body = context_body(&wrapper);
        let encoder = local_call(body, 3);
        assert_eq!(called_name(encoder), "encode");
        assert_eq!(encoder.args.len(), arity);
        let Stmt::Local(reply) = &body.stmts[3] else {
            panic!("expected typed reply")
        };
        let Pat::Type(typed) = &reply.pat else {
            panic!("expected an explicit Vec output type")
        };
        assert_eq!(
            typed.ty.to_token_stream().to_string(),
            ":: std :: vec :: Vec < :: std :: primitive :: u8 >"
        );
        assert_eq!(called_name(last_call(body)), "msg_reply");
    }
}

#[test]
fn sync_guards_precede_context_while_async_guards_precede_spawn_inside_it() {
    let attrs = quote!(
        guard = "first",
        guard = "second",
        decode_with = "decode",
        encode_with = "encode"
    );
    let (_, sync) = functions(
        Kind::Query,
        attrs.clone(),
        quote!(
            fn endpoint() {}
        ),
    );
    assert_eq!(called_name(local_call(&sync.block, 0)), "first");
    assert_eq!(called_name(local_call(&sync.block, 2)), "second");
    assert_eq!(
        called_name(last_call(&sync.block)),
        "in_query_executor_context"
    );
    for index in [1, 3] {
        let Stmt::Expr(Expr::If(check), _) = &sync.block.stmts[index] else {
            panic!("expected ordered refusal check")
        };
        assert!(matches!(
            check.then_branch.stmts.last(),
            Some(Stmt::Expr(Expr::Return(_), _))
        ));
    }
    let (_, asynchronous) = functions(
        Kind::Query,
        attrs,
        quote!(
            async fn endpoint() {}
        ),
    );
    assert_eq!(asynchronous.block.stmts.len(), 1);
    let body = context_body(&asynchronous);
    assert_eq!(called_name(local_call(body, 0)), "first");
    assert_eq!(called_name(local_call(body, 2)), "second");
    let spawn = last_call(body);
    assert_eq!(called_name(spawn), "spawn");
    assert!(
        spawn
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("allow"))
    );
    let Expr::Async(task) = spawn.args.first().unwrap() else {
        panic!("expected one async task")
    };
    assert!(task.capture.is_some());
    assert_eq!(called_name(local_call(&task.block, 0)), "msg_arg_data");
}

#[test]
fn manual_reply_ignores_unused_encoder_path_and_keeps_a_named_result() {
    let (_, wrapper) = functions(
        Kind::Update,
        quote!(
            decode_with = "decode",
            manual_reply,
            encode_with = "not a valid Rust path("
        ),
        quote!(
            fn endpoint(value: impl ::std::fmt::Debug) -> impl ::std::fmt::Debug {
                value
            }
        ),
    );
    let body = context_body(&wrapper);
    assert_eq!(body.stmts.len(), 3);
    let Stmt::Local(result) = body.stmts.last().unwrap() else {
        panic!("expected a named result")
    };
    assert!(matches!(&result.pat, Pat::Ident(binding) if binding.ident == "_result"));
    assert_eq!(
        called_name(call(&result.init.as_ref().unwrap().expr)),
        "endpoint"
    );
}

#[test]
fn lifecycle_and_function_admission_fail_without_panicking() {
    for (kind, attrs, item) in [
        (
            Kind::Update,
            quote!(composite),
            quote!(
                fn endpoint<T>() {}
            ),
        ),
        (
            Kind::Init,
            quote!(manual_reply),
            quote!(
                fn endpoint<T>() {}
            ),
        ),
        (
            Kind::Init,
            quote!(name = ""),
            quote!(
                fn endpoint() {}
            ),
        ),
        (
            Kind::Init,
            quote!(guard = "bad("),
            quote!(
                fn endpoint() {}
            ),
        ),
        (
            Kind::PreUpgrade,
            quote!(),
            quote!(
                fn endpoint(value: u8) {}
            ),
        ),
        (
            Kind::Heartbeat,
            quote!(decode_with = "decode"),
            quote!(
                fn endpoint() {}
            ),
        ),
        (
            Kind::Init,
            quote!(encode_with = "encode"),
            quote!(
                fn endpoint() {}
            ),
        ),
        (
            Kind::PostUpgrade,
            quote!(),
            quote!(
                fn endpoint() -> (()) {
                    ()
                }
            ),
        ),
        (
            Kind::Query,
            quote!(encode_with = "encode"),
            quote!(
                fn endpoint() {}
            ),
        ),
        (
            Kind::Update,
            quote!(decode_with = "decode"),
            quote!(
                fn endpoint() {}
            ),
        ),
        (
            Kind::Query,
            quote!(decode_with = "decode", encode_with = "encode"),
            quote!(
                fn endpoint<T>(value: T) {}
            ),
        ),
        (
            Kind::Query,
            quote!(decode_with = "decode", encode_with = "encode"),
            quote!(
                fn endpoint(&self) {}
            ),
        ),
        (
            Kind::Query,
            quote!(),
            quote!(
                struct NotAFunction;
            ),
        ),
    ] {
        assert!(expand(kind, attrs, item).is_err());
    }
    let (_, wrapper) = functions(
        Kind::Init,
        quote!(manual_reply = false, composite = false),
        quote!(
            fn endpoint() {}
        ),
    );
    let body = context_body(&wrapper);
    assert_eq!(body.stmts.len(), 1);
    let (_, wrapper) = functions(
        Kind::PostUpgrade,
        quote!(decode_with = "decode"),
        quote!(
            fn endpoint() {}
        ),
    );
    assert_eq!(
        called_name(local_call(context_body(&wrapper), 0)),
        "msg_arg_data"
    );
}

#[test]
fn typed_adapter_paths_are_preserved_and_cfg_filters_out_nonconditional_attributes() {
    let attrs = ExportAttributes {
        decode_with: Some(syn::parse_quote!(::codec::decode::<u32>)),
        encode_with: Some(syn::parse_quote!(crate::codec::encode)),
        cratename: Some(syn::parse_quote!(crate::runtime)),
        ..ExportAttributes::default()
    };
    let item = quote!(
        fn endpoint(value: u32) -> u32 {
            value
        }
    );
    let expanded = external_entry(false, attrs, syn::parse2(item.clone()).unwrap(), item).unwrap();
    let file: syn::File = syn::parse2(expanded).unwrap();
    let Item::Fn(wrapper) = &file.items[1] else {
        panic!("expected wrapper")
    };
    let decoder = local_call(context_body(wrapper), 1);
    assert_eq!(
        decoder.func.to_token_stream().to_string(),
        ":: codec :: decode :: < u32 >"
    );

    let item: ItemFn = syn::parse_quote! {
        #[cfg(any())]
        #[cfg_attr(all(), cfg(any()), inline, cfg_attr(any(), cfg(all()), cold))]
        #[doc = "original only"]
        #[allow(unused)]
        fn endpoint() {}
    };
    let conditional = conditional_attributes(&item.attrs).unwrap();
    assert_eq!(conditional.len(), 2);
    let rendered = quote!(#(#conditional)*).to_string();
    assert!(rendered.contains("cfg_attr"));
    assert!(!rendered.contains("inline"));
    assert!(!rendered.contains("cold"));
    assert!(!rendered.contains("doc"));
    assert!(!rendered.contains("allow"));
}
