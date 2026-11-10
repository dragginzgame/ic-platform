mod runtime;

fn decode_one(bytes: Vec<u8>) -> u8 {
    assert_eq!(bytes, [3]);
    runtime::record("decode-one");
    3
}

#[ic_platform_macros::init(crate = "crate::runtime", decode_with = "decode_one")]
fn boot(value: u8) {
    assert_eq!(value, 3);
    runtime::record("init");
}

#[ic_platform_macros::heartbeat(crate = "crate::runtime")]
fn tick() {
    runtime::record("heartbeat");
}

#[ic_platform_macros::pre_upgrade(crate = "crate::runtime")]
fn save() {
    runtime::record("pre-upgrade");
}

#[ic_platform_macros::post_upgrade(crate = "crate::runtime", decode_with = "runtime::decode_unit")]
fn restore() {
    runtime::record("post-upgrade");
}

#[ic_platform_macros::inspect_message(crate = "crate::runtime")]
fn inspect() {
    runtime::record("inspect");
}

#[ic_platform_macros::on_low_wasm_memory(crate = "crate::runtime")]
fn memory() {
    runtime::record("low-memory");
}

#[ic_platform_macros::query(
    crate = "crate::runtime",
    decode_with = "runtime::decode_unit",
    encode_with = "runtime::encode_unit",
    name = "a/b"
)]
fn slash() {
    runtime::record("slash");
}

#[ic_platform_macros::query(
    crate = "crate::runtime",
    decode_with = "runtime::decode_unit",
    encode_with = "runtime::encode_unit",
    name = "a?b"
)]
fn question() {
    runtime::record("question");
}

#[ic_platform_macros::query(
    crate = "crate::runtime",
    decode_with = "runtime::decode_unit",
    encode_with = "runtime::encode_unit",
    name = ""
)]
fn empty() {
    runtime::record("empty");
}

#[ic_platform_macros::query(
    crate = "crate::runtime",
    decode_with = "runtime::decode_unit",
    encode_with = "runtime::encode_unit",
    name = "é"
)]
fn accented() {
    runtime::record("accented");
}

#[ic_platform_macros::query(
    crate = "crate::runtime",
    decode_with = "runtime::decode_unit",
    encode_with = "runtime::encode_unit",
    composite
)]
fn r#type() {
    runtime::record("raw");
}

#[ic_platform_macros::update(
    crate = "crate::runtime",
    decode_with = "runtime::decode_unit",
    encode_with = "runtime::encode_unit"
)]
fn action() {
    runtime::record("action");
}

pub fn check() {
    __canister_method_boot();
    assert_eq!(runtime::take(), ["update", "copy", "decode-one", "init"]);
    __canister_method_tick();
    __canister_method_save();
    __canister_method_restore();
    __canister_method_inspect();
    __canister_method_memory();
    assert_eq!(
        runtime::take(),
        [
            "update",
            "heartbeat",
            "update",
            "pre-upgrade",
            "update",
            "copy",
            "decode",
            "post-upgrade",
            "query",
            "inspect",
            "update",
            "low-memory"
        ]
    );
    __canister_method_slash();
    assert_eq!(
        runtime::take(),
        ["query", "copy", "decode", "slash", "encode", "reply"]
    );
    __canister_method_question();
    assert_eq!(
        runtime::take(),
        ["query", "copy", "decode", "question", "encode", "reply"]
    );
    __canister_method_empty();
    assert_eq!(
        runtime::take(),
        ["query", "copy", "decode", "empty", "encode", "reply"]
    );
    __canister_method_accented();
    assert_eq!(
        runtime::take(),
        ["query", "copy", "decode", "accented", "encode", "reply"]
    );
    __canister_method_type();
    assert_eq!(
        runtime::take(),
        ["query", "copy", "decode", "raw", "encode", "reply"]
    );
    __canister_method_action();
    assert_eq!(
        runtime::take(),
        ["update", "copy", "decode", "action", "encode", "reply"]
    );
}
