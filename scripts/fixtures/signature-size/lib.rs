//! Matched creation/verification probes; no canister runtime or caller policy.
use std::hint::black_box;

#[cfg(feature = "reference-creation")]
use ic_canister_sig_creation as creation;
#[cfg(feature = "owned-creation")]
use ic_platform::canister_sig_creation as creation;

#[cfg(feature = "owned-verification")]
use ic_platform::signature_verification::verify_canister_sig;
#[cfg(feature = "reference-verification")]
use ic_signature_verification::verify_canister_sig;

#[cfg(any(feature = "owned-creation", feature = "reference-creation"))]
#[unsafe(no_mangle)]
pub extern "C" fn key_roundtrip(value: u32) -> u32 {
    let issuer = [0x41; 29];
    #[cfg(feature = "owned-creation")]
    let principal = ic_platform::Principal::from_slice(&issuer);
    #[cfg(feature = "reference-creation")]
    let principal = issuer.as_slice().try_into().unwrap();
    let key = creation::CanisterSigPublicKey::new(principal, vec![value as u8; 32]);
    let der = black_box(key.to_der());
    let decoded = creation::CanisterSigPublicKey::try_from(black_box(der.as_slice())).unwrap();
    let digest = creation::hash_with_domain(b"signature-size", &decoded.to_raw());
    u32::from_le_bytes(digest[..4].try_into().unwrap())
}

#[cfg(any(feature = "owned-verification", feature = "reference-verification"))]
#[unsafe(no_mangle)]
pub extern "C" fn verify(changed: u32) -> u32 {
    let message = if changed == 0 {
        b"message".as_slice()
    } else {
        b"changed".as_slice()
    };
    u32::from(
        verify_canister_sig(
            black_box(message),
            black_box(include_bytes!("signature.bin")),
            black_box(include_bytes!("public.bin")),
            black_box(include_bytes!("root.bin")),
        )
        .is_ok(),
    )
}
