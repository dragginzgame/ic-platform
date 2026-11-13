//! Size probes for reachable Platform capabilities, not application benchmarks.

#[cfg(any(
    feature = "codec-cmc-owned",
    feature = "codec-icp-owned",
    feature = "codec-management-owned",
))]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn protocol_roundtrip() {
    #[cfg(feature = "codec-cmc-owned")]
    use ic_platform::cmc_types::NotifyTopUpResult as Value;
    #[cfg(feature = "codec-icp-owned")]
    use ic_platform::icp_ledger_types::QueryEncodedBlocksResponse as Value;
    #[cfg(feature = "codec-management-owned")]
    use ic_platform::management_canister_types::CanisterStatusResult as Value;
    let bytes = input();
    #[cfg(any(
        feature = "codec-cmc-owned",
        feature = "codec-icp-owned",
        feature = "codec-management-owned"
    ))]
    let output = {
        use ic_platform::candid_codec::{
            decoder::{Limits, decode_one},
            encode_one,
        };
        let value: Value = decode_one(
            &bytes,
            Limits {
                bytes: 1_048_576,
                header: 65_536,
                types: 256,
                work: 10_000_000,
                allocation: 4_194_304,
            },
        )
        .unwrap();
        encode_one(&value, 1_048_576).unwrap()
    };
    reply(&output);
}

#[cfg(any(
    feature = "codec-generic",
    feature = "codec-generic-decode",
    feature = "codec-array-decode",
    feature = "codec-generic-composite",
    feature = "codec-generic-record"
))]
pub use ic_platform::candid_codec as codec;

#[cfg(feature = "codec-array-decode")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn array_decode() {
    let bytes = input();
    let value: [u8; 32] = codec::decoder::decode_one(
        &bytes,
        codec::decoder::Limits {
            bytes: 1_048_576,
            header: 65_536,
            types: 4096,
            work: 10_000_000,
            allocation: 4_194_304,
        },
    )
    .unwrap();
    reply(&value);
}

#[cfg(feature = "codec-generic-record")]
pub mod record_probe;

#[cfg(feature = "codec-generic-composite")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn generic_composite() {
    type Value = (
        Option<Result<Vec<u64>, String>>,
        (num_bigint::BigUint, num_bigint::BigInt),
    );
    let bytes = input();
    let limits = codec::decoder::Limits {
        bytes: 1_048_576,
        header: 65_536,
        types: 4096,
        work: 10_000_000,
        allocation: 4_194_304,
    };
    let value: Value = codec::decoder::decode_one(&bytes, limits).unwrap();
    reply(&codec::encode_one(&value, 1_048_576).unwrap());
}

#[cfg(feature = "codec-generic-decode")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn generic_decode_text() {
    let bytes = input();
    let limits = codec::decoder::Limits {
        bytes: 1_048_576,
        header: 65_536,
        types: 4096,
        work: 10_000_000,
        allocation: 4_194_304,
    };
    let text: String = codec::decoder::decode_one(&bytes, limits).unwrap();
    reply(text.as_bytes());
}

#[cfg(feature = "codec-generic")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn generic_encode_text() {
    let bytes = input();
    let text = std::str::from_utf8(&bytes).unwrap();
    reply(&codec::encode_one(text, usize::MAX).unwrap());
}

// The isolated raw Runtime profile uses CDK I/O directly; the other profiles
// share these byte helpers. The runner selects exactly one profile at a time.
#[cfg(not(feature = "runtime"))]
fn input() -> Vec<u8> {
    let mut bytes = vec![0; ic_platform::ic0::msg_arg_data_size()];
    ic_platform::ic0::msg_arg_data_copy(&mut bytes, 0);
    bytes
}

#[cfg(not(feature = "runtime"))]
fn reply(bytes: &[u8]) {
    ic_platform::ic0::msg_reply_data_append(bytes);
    ic_platform::ic0::msg_reply();
}

// Dependency presence control: identical reachable code with/without Candid.
#[cfg(feature = "codec-empty")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn codec_control() {
    reply(&input());
}

// Matched input admission/output framing; only the encoder differs.
#[cfg(feature = "codec-text-owned")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn codec_text() {
    let bytes = input();
    let text = std::str::from_utf8(&bytes).unwrap();
    #[cfg(feature = "codec-text-owned")]
    let encoded = ic_platform::candid_args::text(text);
    reply(&encoded);
}

#[cfg(feature = "candid-args")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query encode_units")]
pub extern "C" fn encode_units() {
    reply(&ic_platform::candid_args::units(input().len()));
}

#[cfg(feature = "candid-args")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query encode_text")]
pub extern "C" fn encode_text() {
    let bytes = input();
    reply(&ic_platform::candid_args::text(
        std::str::from_utf8(&bytes).unwrap(),
    ));
}

// Opt-in IC instruction probes. Both arms use these exact same fixture bodies;
// counters exclude request admission and reply framing.
#[cfg(feature = "runtime-system")]
fn measure_system_read(read: fn() -> Vec<u8>) {
    let request = input();
    let repetitions = u32::from_le_bytes(request.as_slice().try_into().unwrap());
    assert!((1..=64).contains(&repetitions));
    let start = ic_platform::cdk::api::instruction_counter();
    let first = read();
    for _ in 1..repetitions {
        assert_eq!(read(), first);
    }
    let instructions = ic_platform::cdk::api::instruction_counter() - start;
    let mut response = instructions.to_le_bytes().to_vec();
    response.extend_from_slice(&first);
    reply(&response);
}

#[cfg(feature = "runtime-system")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query system_certificate")]
pub extern "C" fn system_certificate() {
    measure_system_read(|| ic_platform::cdk::api::data_certificate().unwrap());
}

#[cfg(feature = "runtime-system")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_inspect_message")]
pub extern "C" fn system_inspect() {
    assert_eq!(ic_platform::cdk::api::msg_method_name(), "system_root_key");
    ic_platform::cdk::api::accept_message();
}

#[cfg(feature = "runtime-system")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_update system_root_key")]
pub extern "C" fn system_root_key() {
    measure_system_read(ic_platform::cdk::api::root_key);
}

#[cfg(feature = "certification-cbor")]
/// Decode the supplied hash tree under the strict external-protocol byte bound.
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query decode_tree")]
pub extern "C" fn decode_tree() {
    let bytes = input();
    let result = ic_platform::certification::cbor::strict::decode_hash_tree(
        &bytes,
        ic_platform::certification::cbor::strict::DecodeLimits { bytes: bytes.len() },
    );
    reply(&[u8::from(result.is_ok())]);
}

#[cfg(feature = "ecdsa")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query verify_ecdsa")]
pub extern "C" fn verify_ecdsa() {
    use ic_platform::ecdsa::{PrehashVerifier, Signature, VerifyingKey};
    let bytes = input();
    let (key, rest) = bytes.split_at(33);
    let (prehash, signature) = rest.split_at(32);
    #[cfg(feature = "ecdsa-instructions")]
    let start = ic_platform::ic0::performance_counter(0);
    let valid = VerifyingKey::from_sec1_bytes(key).and_then(|key| {
        let signature = Signature::from_slice(signature)?;
        key.verify_prehash(prehash, &signature)
    });
    #[cfg(feature = "ecdsa-instructions")]
    {
        let instructions = ic_platform::ic0::performance_counter(0) - start;
        let mut result = [0; 9];
        result[0] = u8::from(valid.is_ok());
        result[1..].copy_from_slice(&instructions.to_le_bytes());
        reply(&result);
    }
    #[cfg(not(feature = "ecdsa-instructions"))]
    reply(&[u8::from(valid.is_ok())]);
}

// Signature/public-key lengths are fixed by the upstream BLS encoding. The
// remainder is caller-supplied message data; measure verification, not framing.
#[cfg(feature = "bls")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query verify_bls")]
pub extern "C" fn verify_bls() {
    run_bls();
}

#[cfg(feature = "bls")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_update verify_bls_update")]
pub extern "C" fn verify_bls_update() {
    run_bls();
}

#[cfg(feature = "bls")]
fn run_bls() {
    let bytes = input();
    let (signature, rest) = bytes.split_at(48);
    let (key, message) = rest.split_at(96);
    let start = ic_platform::ic0::performance_counter(0);
    let valid = ic_platform::bls::verify_bls_signature(signature, message, key).is_ok();
    let instructions = ic_platform::ic0::performance_counter(0) - start;
    let mut result = [0; 9];
    result[0] = u8::from(valid);
    result[1..].copy_from_slice(&instructions.to_le_bytes());
    reply(&result);
}

#[cfg(feature = "principal")]
mod principal {
    use super::*;
    use ic_platform::{Account, Principal};

    fn finish(result: Result<Principal, ic_platform::PrincipalError>) {
        match result {
            Ok(owner) => {
                let account = Account {
                    owner,
                    subaccount: Some([42; 32]),
                };
                let json = serde_json::to_vec(&account).unwrap();
                let text = account.to_string();
                reply(&json);
                // Keep Account text formatting independently reachable.
                ic_platform::ic0::debug_print(text.as_bytes());
            }
            Err(error) => reply(error.to_string().as_bytes()),
        }
    }

    #[expect(
        unsafe_code,
        reason = "The isolated Wasm probe exports its IC entrypoint ABI"
    )]
    #[unsafe(export_name = "canister_query parse_slice")]
    pub extern "C" fn parse_slice() {
        let bytes = input();
        finish(Principal::from_text(std::str::from_utf8(&bytes).unwrap()));
    }

    #[expect(
        unsafe_code,
        reason = "The isolated Wasm probe exports its IC entrypoint ABI"
    )]
    #[unsafe(export_name = "canister_query parse_string")]
    pub extern "C" fn parse_string() {
        finish(Principal::from_text(String::from_utf8(input()).unwrap()));
    }

    #[expect(
        unsafe_code,
        reason = "The isolated Wasm probe exports its IC entrypoint ABI"
    )]
    #[unsafe(export_name = "canister_query parse_string_ref")]
    pub extern "C" fn parse_string_ref() {
        let text = String::from_utf8(input()).unwrap();
        finish(Principal::from_text(&text));
    }
}

#[cfg(feature = "runtime")]
/// Submit a raw call to this canister and forward its response or rejection.
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_update raw_call")]
pub extern "C" fn raw_call() {
    use ic_platform::cdk::{api, call::Call, futures};
    futures::internals::in_executor_context(|| {
        futures::spawn(async {
            let bytes = api::msg_arg_data();
            match Call::bounded_wait(api::canister_self(), "probe")
                .take_raw_args(bytes)
                .await
            {
                Ok(response) => api::msg_reply(response),
                Err(error) => api::msg_reject(error.to_string()),
            }
        });
    });
}

#[cfg(feature = "verification")]
/// Verify a signature using the fixture's equal-quarter input framing.
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query verify")]
pub extern "C" fn verify() {
    let bytes = input();
    let quarter = bytes.len() / 4;
    let (message, rest) = bytes.split_at(quarter);
    let (signature, rest) = rest.split_at(quarter);
    let (key, root) = rest.split_at(quarter);
    let result =
        ic_platform::signature_verification::verify_canister_sig(message, signature, key, root);
    reply(&[u8::from(result.is_ok())]);
}

#[cfg(feature = "principal-binary")]
/// Return an admitted Principal's canonical bytes, or an empty reply on refusal.
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query binary_principal")]
pub extern "C" fn binary_principal() {
    use ic_platform::{Account, Principal};
    match Principal::try_from_slice(&input()) {
        Ok(owner) => reply(Account::from(owner).owner.as_slice()),
        Err(_) => reply(&[]),
    }
}

#[cfg(feature = "principal-self-auth")]
/// Return the self-authenticating Principal for the supplied slice.
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query self_auth_slice")]
pub extern "C" fn self_auth_slice() {
    let bytes = input();
    reply(ic_platform::Principal::self_authenticating(bytes.as_slice()).as_slice());
}

#[cfg(feature = "principal-self-auth")]
/// Return the self-authenticating Principal using an owned request vector.
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query self_auth_vec")]
pub extern "C" fn self_auth_owned() {
    reply(ic_platform::Principal::self_authenticating(input()).as_slice());
}

#[cfg(feature = "principal-self-auth")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query self_auth_vec_ref")]
/// Return the self-authenticating Principal using a borrowed request vector.
pub extern "C" fn self_auth_borrowed() {
    let bytes = input();
    reply(ic_platform::Principal::self_authenticating(&bytes).as_slice());
}

// Exercise both ledger hashes and self-authentication to measure provider sharing.
#[cfg(feature = "ledger-hash")]
#[expect(
    unsafe_code,
    reason = "The isolated Wasm probe exports its IC entrypoint ABI"
)]
#[unsafe(export_name = "canister_query ledger_hash")]
pub extern "C" fn ledger_hash() {
    use ic_platform::{Principal, icp_ledger_types::account_identifier, icrc_ledger_types::Value};
    let bytes = input();
    let principal = Principal::self_authenticating(&bytes);
    let mut output = account_identifier(principal, &[0; 32]).to_vec();
    output.extend_from_slice(&Value::Blob(bytes.into()).hash());
    reply(&output);
}
