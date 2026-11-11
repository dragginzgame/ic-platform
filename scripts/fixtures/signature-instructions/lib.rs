//! Installed measurement adapter only; request/reply bytes are fixture-owned.
#[cfg(not(feature = "owned"))]
use ic_canister_sig_creation as creation;
#[cfg(feature = "owned")]
use ic_platform::canister_sig_creation as creation;
#[cfg(all(feature = "owned", not(feature = "owned-prepared")))]
use ic_platform::signature_verification::verify_canister_sig;
use ic_platform_runtime::ic0;
#[cfg(feature = "reference")]
use ic_signature_verification::verify_canister_sig;
use std::hint::black_box;
#[cfg(feature = "owned-prepared")]
static PREPARED_ROOT: std::sync::LazyLock<ic_platform::bls::PreparedPublicKey> =
    std::sync::LazyLock::new(|| {
        ic_platform::bls::PublicKey::deserialize(include_bytes!("root.bin"))
            .unwrap()
            .prepare()
    });
#[cfg(feature = "reference-admitted")]
mod admitted;
#[cfg(feature = "reference-admitted")]
use admitted::verify_canister_sig;

#[unsafe(export_name = "canister_update measure")]
pub extern "C" fn measure() {
    let mut input = vec![0; ic0::msg_arg_data_size()];
    ic0::msg_arg_data_copy(&mut input, 0);
    let code = input[0];
    let repetitions = u32::from_le_bytes(input[1..5].try_into().unwrap());
    let issuer = [0x5b; 29];
    #[cfg(feature = "owned")]
    let principal = ic_platform::Principal::from_slice(&issuer);
    #[cfg(not(feature = "owned"))]
    let principal = issuer.as_slice().try_into().unwrap();
    let key = creation::CanisterSigPublicKey::new(principal, vec![0x39; 32]);
    let raw = key.to_raw();
    let der = key.to_der();
    let payload = vec![0xc9; 4096];
    let signature: &[u8] = match code {
        10 | 13 | 14 => include_bytes!("direct.bin"),
        11 => include_bytes!("legacy.bin"),
        _ => include_bytes!("sharded.bin"),
    };
    let mut root = include_bytes!("root.bin").to_vec();
    if code == 14 {
        root.fill(0);
        root[0] = 0xc0;
    }
    let message: &[u8] = if code == 13 { b"changed" } else { b"message" };
    let start = ic0::performance_counter(0);
    let mut success = true;
    for _ in 0..repetitions {
        success &= match code {
            0 => black_box(true),
            1 => {
                black_box(black_box(&key).to_raw());
                true
            }
            2 => {
                black_box(black_box(&key).to_der());
                true
            }
            3 => black_box(creation::CanisterSigPublicKey::try_from_raw(black_box(
                &raw,
            )))
            .is_ok(),
            4 => black_box(creation::CanisterSigPublicKey::try_from(black_box(
                der.as_slice(),
            )))
            .is_ok(),
            5 => {
                black_box(creation::hash_bytes(black_box(&payload[..32])));
                true
            }
            6 => {
                black_box(creation::hash_bytes(black_box(&payload)));
                true
            }
            _ => black_box(verify(
                black_box(message),
                black_box(signature),
                black_box(include_bytes!("public.bin")),
                black_box(&root),
            ))
            .is_ok(),
        };
    }
    let instructions = ic0::performance_counter(0) - start;
    let mut output = [0; 9];
    output[0] = u8::from(success);
    output[1..].copy_from_slice(&instructions.to_le_bytes());
    ic0::msg_reply_data_append(&output);
    ic0::msg_reply();
}

fn verify(message: &[u8], signature: &[u8], public: &[u8], root: &[u8]) -> Result<(), String> {
    #[cfg(feature = "owned-prepared")]
    {
        let alternate;
        let prepared = if root == include_bytes!("root.bin") {
            &*PREPARED_ROOT
        } else {
            alternate = ic_platform::bls::PublicKey::deserialize(root)
                .unwrap()
                .prepare();
            &alternate
        };
        ic_platform::signature_verification::verify_canister_sig_with_prepared_root(
            message, signature, public, prepared,
        )
    }
    #[cfg(not(feature = "owned-prepared"))]
    verify_canister_sig(message, signature, public, root)
}
