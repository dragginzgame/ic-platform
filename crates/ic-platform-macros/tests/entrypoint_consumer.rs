//! Specification-only observations of generated native entrypoint wrappers.

use ic_platform_macros::{
    heartbeat, init, inspect_message, on_low_wasm_memory, post_upgrade, pre_upgrade, query, update,
};

mod runtime {
    use std::cell::RefCell;

    thread_local! {
        static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    pub fn record(event: impl Into<String>) {
        EVENTS.with(|events| events.borrow_mut().push(event.into()));
    }

    pub fn take() -> Vec<String> {
        EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
    }

    pub mod api {
        pub fn msg_arg_data() -> Vec<u8> {
            super::record("copy");
            vec![7, 8]
        }

        pub fn msg_reply(bytes: Vec<u8>) {
            super::record(format!("reply:{bytes:?}"));
        }

        pub fn msg_reject(message: &str) {
            super::record(format!("reject:{message}"));
        }
    }

    pub mod futures {
        use std::future::Future;
        use std::pin::pin;
        use std::task::{Context, Waker};

        pub fn spawn(future: impl Future<Output = ()> + 'static) {
            super::record("spawn");
            let mut future = pin!(future);
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_ready(),
                "fixture handlers must be immediately ready"
            );
            super::record("spawn:done");
        }

        pub mod internals {
            pub fn in_executor_context(action: impl FnOnce()) {
                super::super::record("update:start");
                action();
                super::super::record("update:end");
            }

            pub fn in_query_executor_context(action: impl FnOnce()) {
                super::super::record("query:start");
                action();
                super::super::record("query:end");
            }
        }
    }
}

fn decode_unit(bytes: Vec<u8>) {
    assert_eq!(bytes, [7, 8]);
    runtime::record("decode:unit");
}

fn encode_unit() -> Vec<u8> {
    runtime::record("encode:unit");
    vec![0]
}

fn guard_first() -> Result<(), String> {
    runtime::record("guard:first");
    Ok(())
}

fn guard_second() -> Result<(), String> {
    runtime::record("guard:second");
    Ok(())
}

fn guard_refuse() -> Result<(), String> {
    runtime::record("guard:refuse");
    Err("denied\0λ".to_owned())
}

#[query(
    crate = "crate::runtime",
    decode_with = "decode_unit",
    encode_with = "encode_unit",
    guard = "guard_first",
    guard = "guard_second",
    composite,
    name = "unit !☃"
)]
fn r#type() {
    runtime::record("handler:unit");
}

mod codecs {
    pub fn one<T: From<u8>>(bytes: Vec<u8>) -> T {
        assert_eq!(bytes, [7, 8]);
        super::runtime::record("decode:one");
        T::from(7)
    }

    pub fn scalar<T: Into<u64>>(value: T) -> Vec<u8> {
        let value = value.into();
        super::runtime::record(format!("encode:scalar:{value}"));
        vec![u8::try_from(value).unwrap()]
    }

    pub fn tuple_argument(bytes: Vec<u8>) -> (u32, u32) {
        assert_eq!(bytes, [7, 8]);
        super::runtime::record("decode:tuple-argument");
        (7, 8)
    }

    pub fn multiple(bytes: Vec<u8>) -> ((u32, u32), u32, u32) {
        assert_eq!(bytes, [7, 8]);
        super::runtime::record("decode:multiple");
        ((1, 2), 3, 4)
    }

    pub fn tuple_result(value: (u32, u32)) -> Vec<u8> {
        super::runtime::record(format!("encode:tuple:{value:?}"));
        vec![
            u8::try_from(value.0).unwrap(),
            u8::try_from(value.1).unwrap(),
        ]
    }
}

#[update(
    crate = "crate::runtime",
    decode_with = "codecs::one::<u32>",
    encode_with = "codecs::scalar::<u32>",
    composite = false,
    manual_reply = false
)]
fn scalar(mut value: u32) -> u32 {
    value += 1;
    runtime::record("handler:scalar");
    value
}

#[query(
    crate = "crate::runtime",
    decode_with = "::std::convert::identity::<::std::vec::Vec<u8>>",
    encode_with = "::std::convert::identity::<::std::vec::Vec<u8>>"
)]
async fn absolute_paths(value: Vec<u8>) -> Vec<u8> {
    runtime::record("handler:absolute-paths");
    value
}

#[query(
    crate = "crate::runtime",
    decode_with = "crate::codecs::tuple_argument",
    encode_with = "crate::codecs::tuple_result"
)]
fn single_tuple((left, mut right): (u32, u32)) -> (u32, u32) {
    right += 1;
    runtime::record("handler:tuple");
    (left, right)
}

#[update(
    crate = "crate::runtime",
    decode_with = "codecs::multiple",
    encode_with = "codecs::tuple_result",
    guard = "guard_first",
    guard = "guard_second"
)]
async fn multiple((_, left): (u32, u32), mut right: u32, _: u32) -> (u32, u32) {
    runtime::record("handler:async");
    std::future::ready(()).await;
    right += 1;
    runtime::record("handler:ready");
    (left, right)
}

#[query(
    crate = "crate::runtime",
    decode_with = "decode_unit",
    encode_with = "encode_unit",
    guard = "guard_first",
    guard = "guard_refuse",
    guard = "guard_second"
)]
fn refused_sync() {
    runtime::record("unexpected:handler");
}

#[update(
    crate = "crate::runtime",
    decode_with = "decode_unit",
    encode_with = "encode_unit",
    guard = "guard_refuse",
    guard = "guard_second"
)]
async fn refused_async() {
    runtime::record("unexpected:handler");
}

#[update(
    crate = "crate::runtime",
    decode_with = "decode_unit",
    manual_reply,
    encode_with = "not a path !!!"
)]
async fn manual() {
    runtime::record("handler:manual");
    runtime::api::msg_reply(vec![91]);
}

#[init(crate = "crate::runtime")]
fn initialize() {
    runtime::record("handler:init");
}

#[post_upgrade(crate = "crate::runtime", decode_with = "codecs::multiple")]
fn restore((_, left): (u32, u32), right: u32, _: u32) {
    assert_eq!((left, right), (2, 3));
    runtime::record("handler:post-upgrade");
}

#[pre_upgrade(crate = "crate::runtime")]
fn save() {
    runtime::record("handler:pre-upgrade");
}

#[heartbeat(crate = "crate::runtime")]
async fn beat() {
    runtime::record("handler:heartbeat");
}

#[inspect_message(crate = "crate::runtime")]
fn inspect() {
    runtime::record("handler:inspect");
}

#[on_low_wasm_memory(crate = "crate::runtime")]
async fn memory() {
    runtime::record("handler:low-memory");
}

#[query(
    crate = "crate::runtime",
    decode_with = "this_path_does_not_exist",
    encode_with = "neither_does_this"
)]
#[cfg(any())]
fn disabled_after_attribute() {}

mod aliases {
    type Unit = ();

    fn decode(bytes: Vec<u8>) {
        super::decode_unit(bytes);
    }

    fn encode(_value: Unit) -> Vec<u8> {
        super::runtime::record("encode:unit-alias-as-scalar");
        vec![12]
    }

    #[ic_platform_macros::query(
        crate = "crate::runtime",
        decode_with = "decode",
        encode_with = "encode"
    )]
    fn aliased_unit() -> Unit {
        super::runtime::record("handler:unit-alias");
    }

    pub fn invoke() {
        __canister_method_aliased_unit();
    }
}

mod reviewed_regressions {
    fn decode_concrete(bytes: Vec<u8>) -> u32 {
        assert_eq!(bytes, [7, 8]);
        super::runtime::record("decode:concrete");
        7
    }

    #[ic_platform_macros::update(
        crate = "crate::runtime",
        decode_with = "decode_concrete",
        encode_with = "crate::codecs::scalar"
    )]
    fn opaque(value: impl Into<u64>) -> impl Into<u64> {
        super::runtime::record("handler:opaque");
        value.into() + 1
    }

    fn __ic_result(value: u32) -> Vec<u8> {
        assert_eq!(value, 8);
        super::runtime::record("encode:collision");
        vec![8]
    }

    #[ic_platform_macros::query(
        crate = "crate::runtime",
        decode_with = "decode_concrete",
        encode_with = "__ic_result"
    )]
    fn __ic_argument_0(__ic_result: u32) -> u32 {
        super::runtime::record("handler:collision");
        __ic_result + 1
    }

    pub fn invoke_opaque() {
        __canister_method_opaque();
    }

    pub fn invoke_collision() {
        __canister_method___ic_argument_0();
    }
}

#[expect(
    dead_code,
    reason = "Prelude names deliberately shadow generated references"
)]
mod hygiene {
    struct Vec;
    struct String;
    enum Result {}
    struct Ok;
    struct Err;

    fn guard_result() -> ::std::result::Result<(), ::std::string::String> {
        super::runtime::record("guard:hygiene");
        ::std::result::Result::Ok(())
    }

    fn args(bytes: ::std::vec::Vec<u8>) -> u32 {
        assert_eq!(bytes, [7, 8]);
        super::runtime::record("decode:hygiene");
        22
    }

    fn bytes<T: From<::std::vec::Vec<u8>>>(value: u32) -> T {
        assert_eq!(value, 22);
        super::runtime::record("encode:hygiene");
        ::std::vec![22].into()
    }

    #[ic_platform_macros::query(
        crate = "crate::runtime",
        guard = "guard_result",
        decode_with = "args",
        encode_with = "bytes"
    )]
    fn result(__arg_0: u32) -> u32 {
        super::runtime::record("handler:hygiene");
        __arg_0
    }

    pub fn invoke() {
        __canister_method_result();
    }
}

#[test]
fn synchronous_guards_precede_context_and_unit_codecs_run() {
    __canister_method_type();
    assert_eq!(
        runtime::take(),
        [
            "guard:first",
            "guard:second",
            "query:start",
            "copy",
            "decode:unit",
            "handler:unit",
            "encode:unit",
            "reply:[0]",
            "query:end"
        ]
    );
}

#[test]
fn scalar_and_single_tuple_arguments_preserve_shapes_and_patterns() {
    __canister_method_scalar();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "copy",
            "decode:one",
            "handler:scalar",
            "encode:scalar:8",
            "reply:[8]",
            "update:end"
        ]
    );
    __canister_method_single_tuple();
    assert_eq!(
        runtime::take(),
        [
            "query:start",
            "copy",
            "decode:tuple-argument",
            "handler:tuple",
            "encode:tuple:(7, 9)",
            "reply:[7, 9]",
            "query:end"
        ]
    );
}

#[test]
fn asynchronous_context_precedes_guards_and_spawn_owns_decoding() {
    __canister_method_multiple();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "guard:first",
            "guard:second",
            "spawn",
            "copy",
            "decode:multiple",
            "handler:async",
            "handler:ready",
            "encode:tuple:(2, 4)",
            "reply:[2, 4]",
            "spawn:done",
            "update:end"
        ]
    );
}

#[test]
fn asynchronous_query_accepts_absolute_generic_codec_paths() {
    __canister_method_absolute_paths();
    assert_eq!(
        runtime::take(),
        [
            "query:start",
            "spawn",
            "copy",
            "handler:absolute-paths",
            "reply:[7, 8]",
            "spawn:done",
            "query:end"
        ]
    );
}

#[test]
fn guard_refusal_stops_each_execution_path() {
    __canister_method_refused_sync();
    assert_eq!(
        runtime::take(),
        ["guard:first", "guard:refuse", "reject:denied\0λ"]
    );
    __canister_method_refused_async();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "guard:refuse",
            "reject:denied\0λ",
            "update:end"
        ]
    );
}

#[test]
fn manual_reply_ignores_the_unused_encoder_path() {
    __canister_method_manual();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "spawn",
            "copy",
            "decode:unit",
            "handler:manual",
            "reply:[91]",
            "spawn:done",
            "update:end"
        ]
    );
}

#[test]
fn every_lifecycle_kind_selects_its_context_and_codec_policy() {
    __canister_method_initialize();
    assert_eq!(
        runtime::take(),
        ["update:start", "handler:init", "update:end"]
    );
    __canister_method_restore();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "copy",
            "decode:multiple",
            "handler:post-upgrade",
            "update:end"
        ]
    );
    __canister_method_save();
    assert_eq!(
        runtime::take(),
        ["update:start", "handler:pre-upgrade", "update:end"]
    );
    __canister_method_beat();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "spawn",
            "handler:heartbeat",
            "spawn:done",
            "update:end"
        ]
    );
    __canister_method_inspect();
    assert_eq!(
        runtime::take(),
        ["query:start", "handler:inspect", "query:end"]
    );
    __canister_method_memory();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "spawn",
            "handler:low-memory",
            "spawn:done",
            "update:end"
        ]
    );
}

#[test]
fn alias_shape_generic_encoder_inference_and_hygiene_are_preserved() {
    aliases::invoke();
    assert_eq!(
        runtime::take(),
        [
            "query:start",
            "copy",
            "decode:unit",
            "handler:unit-alias",
            "encode:unit-alias-as-scalar",
            "reply:[12]",
            "query:end"
        ]
    );
    hygiene::invoke();
    assert_eq!(
        runtime::take(),
        [
            "guard:hygiene",
            "query:start",
            "copy",
            "decode:hygiene",
            "handler:hygiene",
            "encode:hygiene",
            "reply:[22]",
            "query:end"
        ]
    );
}

#[test]
fn impl_trait_handler_uses_concrete_decode_and_generic_encode() {
    reviewed_regressions::invoke_opaque();
    assert_eq!(
        runtime::take(),
        [
            "update:start",
            "copy",
            "decode:concrete",
            "handler:opaque",
            "encode:scalar:8",
            "reply:[8]",
            "update:end"
        ]
    );
}

#[test]
fn generated_local_spellings_cannot_capture_handler_or_encoder() {
    reviewed_regressions::invoke_collision();
    assert_eq!(
        runtime::take(),
        [
            "query:start",
            "copy",
            "decode:concrete",
            "handler:collision",
            "encode:collision",
            "reply:[8]",
            "query:end"
        ]
    );
}
