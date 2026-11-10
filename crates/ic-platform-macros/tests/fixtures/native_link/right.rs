mod runtime;

#[ic_platform_macros::init(crate = "crate::runtime", decode_with = "runtime::decode_unit")]
fn boot() {
    runtime::record("init-zero-decoded");
}

#[ic_platform_macros::heartbeat(crate = "crate::runtime")]
fn tick() {
    runtime::record("heartbeat");
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

pub fn check() {
    __canister_method_boot();
    assert_eq!(
        runtime::take(),
        ["update", "copy", "decode", "init-zero-decoded"]
    );
    __canister_method_tick();
    assert_eq!(runtime::take(), ["update", "heartbeat"]);
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
}
