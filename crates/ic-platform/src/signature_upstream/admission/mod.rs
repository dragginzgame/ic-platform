//! Owned framing admission before the unchanged imported key parser.
//! No key encoding, cryptography, seed limit or issuer policy is implemented here.

pub(super) fn canister_sig_der(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 20 {
        return Err("canister sig pk shorter than DER prefix".into());
    }
    // The pinned encoder/parser use the short-form SubjectPublicKeyInfo grammar.
    // Long-form lengths are not supported by that parser's fixed raw-key offset.
    if bytes[2..16] != super::creation::CANISTER_SIG_PK_DER_OID[..] {
        return Err("invalid OID of canister sig pk".into());
    }
    if bytes[1] & 0x80 != 0
        || usize::from(bytes[1]) != bytes.len() - 2
        || bytes[17] & 0x80 != 0
        || usize::from(bytes[17]) != bytes.len() - 18
    {
        return Err("canister sig pk too short or invalid DER length".into());
    }
    if bytes[0] != 0x30 || bytes[16] != 0x03 || bytes[18] != 0 {
        return Err("invalid canister sig pk DER envelope".into());
    }
    Ok(())
}
