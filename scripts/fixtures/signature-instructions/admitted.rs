//! Benchmark reference adapter for Platform's additional identity-key admission.
//! This is not a production verifier or a replacement cryptographic primitive.
use ic_certification::{Certificate, LookupResult};
use serde::Deserialize;

#[derive(Deserialize)]
struct Envelope {
    #[serde(with = "serde_bytes")]
    certificate: Vec<u8>,
}

fn admit(bytes: &[u8]) -> Result<(), String> {
    let encoded: &[u8; 96] = bytes.try_into().map_err(|_| "invalid BLS signature")?;
    let point =
        Option::<ic_bls12_381::G2Affine>::from(ic_bls12_381::G2Affine::from_compressed(encoded))
            .ok_or("invalid BLS signature")?;
    if bool::from(point.is_identity()) {
        Err("invalid BLS signature".into())
    } else {
        Ok(())
    }
}

pub fn verify_canister_sig(
    message: &[u8],
    signature: &[u8],
    public: &[u8],
    root: &[u8],
) -> Result<(), String> {
    ic_signature_verification::verify_canister_sig(message, signature, public, root)?;
    admit(root)?;
    let envelope: Envelope = serde_cbor::from_slice(signature).map_err(|e| e.to_string())?;
    let certificate: Certificate =
        serde_cbor::from_slice(&envelope.certificate).map_err(|e| e.to_string())?;
    if let Some(delegation) = certificate.delegation {
        let authority: Certificate =
            serde_cbor::from_slice(&delegation.certificate).map_err(|e| e.to_string())?;
        let path = [
            b"subnet".as_slice(),
            delegation.subnet_id.as_slice(),
            b"public_key".as_slice(),
        ];
        let LookupResult::Found(der) = authority.tree.lookup_path(path) else {
            return Err("missing delegated key".into());
        };
        admit(&ic_canister_sig_creation::extract_raw_root_pk_from_der(
            der,
        )?)?;
    }
    Ok(())
}
