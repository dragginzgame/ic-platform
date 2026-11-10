//! Canister-signature public-key framing and protocol hash inputs.
//!
//! SHA-256 is supplied by `sha2`; callers own seeds, witnesses and signature
//! publication. This module does not maintain a signature store.

use crate::principal::Principal;
use sha2_11::{Digest, Sha256};

const CANISTER_ALGORITHM: &[u8; 14] = b"\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\x83\xb8\x43\x01\x02";

/// The network's DER BLS algorithm and bit-string framing.
pub const IC_ROOT_PK_DER_PREFIX: &[u8; 37] = b"\x30\x81\x82\x30\x1d\x06\x0d\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x01\x02\x01\x06\x0c\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x02\x01\x03\x61\x00";

/// Mainnet root public key in DER framing; verification never selects it implicitly.
pub const IC_ROOT_PK_DER: &[u8; 133] = b"\x30\x81\x82\x30\x1d\x06\x0d\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x01\x02\x01\x06\x0c\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x02\x01\x03\x61\x00\x81\x4c\x0e\x6e\xc7\x1f\xab\x58\x3b\x08\xbd\x81\x37\x3c\x25\x5c\x3c\x37\x1b\x2e\x84\x86\x3c\x98\xa4\xf1\xe0\x8b\x74\x23\x5d\x14\xfb\x5d\x9c\x0c\xd5\x46\xd9\x68\x5f\x91\x3a\x0c\x0b\x2c\xc5\x34\x15\x83\xbf\x4b\x43\x92\xe4\x67\xdb\x96\xd6\x5b\x9b\xb4\xcb\x71\x71\x12\xf8\x47\x2e\x0d\x5a\x4d\x14\x50\x5f\xfd\x74\x84\xb0\x12\x91\x09\x1c\x5f\x87\xb9\x88\x83\x46\x3f\x98\x09\x1a\x0b\xaa\xae";

/// Issuing canister and application-selected seed of a canister signature key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanisterSigPublicKey {
    /// Canonical identity of the issuing canister.
    pub canister_id: Principal,
    /// Opaque seed, without a size policy imposed by this API.
    pub seed: Vec<u8>,
}

impl CanisterSigPublicKey {
    /// Retains the issuer and seed without changing their bytes.
    pub fn new(canister_id: Principal, seed: Vec<u8>) -> Self {
        Self { canister_id, seed }
    }

    /// Decodes a length-prefixed issuer followed by the remaining seed bytes.
    pub fn try_from_raw(raw: &[u8]) -> Result<Self, String> {
        let (&issuer_len, body) = raw.split_first().ok_or("empty raw canister sig pk")?;
        let issuer_len = usize::from(issuer_len);
        let issuer = body.get(..issuer_len).ok_or("canister sig pk too short")?;
        let canister_id = Principal::try_from_slice(issuer)
            .map_err(|error| format!("invalid canister id in canister sig pk: {error}"))?;
        Ok(Self::new(canister_id, body[issuer_len..].to_vec()))
    }

    /// Encodes one issuer-length byte, issuer bytes, then seed bytes.
    pub fn to_raw(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + self.canister_id.as_slice().len() + self.seed.len());
        bytes.push(self.canister_id.len());
        bytes.extend_from_slice(self.canister_id.as_slice());
        bytes.extend_from_slice(&self.seed);
        bytes
    }

    /// Encodes the historical one-byte-length DER envelope.
    ///
    /// For compatibility this infallible encoder truncates length fields to a
    /// byte even for large seeds. Such output need not pass strict DER admission.
    pub fn to_der(&self) -> Vec<u8> {
        let raw = self.to_raw();
        let mut bytes = Vec::with_capacity(19 + raw.len());
        bytes.extend_from_slice(&[0x30, (17 + raw.len()) as u8]);
        bytes.extend_from_slice(CANISTER_ALGORITHM);
        bytes.extend_from_slice(&[0x03, (1 + raw.len()) as u8, 0]);
        bytes.extend_from_slice(&raw);
        bytes
    }
}

impl TryFrom<&[u8]> for CanisterSigPublicKey {
    type Error = String;

    /// Admits only complete, byte-aligned, short-form DER key envelopes.
    fn try_from(der: &[u8]) -> Result<Self, Self::Error> {
        if der.len() < 20 {
            return Err("canister sig pk shorter than DER prefix".into());
        }
        if &der[2..16] != CANISTER_ALGORITHM {
            return Err("invalid OID of canister sig pk".into());
        }
        if der[1] >= 128
            || der[17] >= 128
            || usize::from(der[1]) != der.len() - 2
            || usize::from(der[17]) != der.len() - 18
        {
            return Err("canister sig pk too short or invalid DER length".into());
        }
        if der[0] != 0x30 || der[16] != 0x03 || der[18] != 0 {
            return Err("invalid canister sig pk DER envelope".into());
        }
        Self::try_from_raw(&der[19..])
    }
}

/// Computes SHA-256 using the external implementation.
pub fn hash_bytes(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    Sha256::digest(bytes.as_ref()).into()
}

/// Hashes the one-byte domain length, domain bytes, then payload bytes.
///
/// The length byte retains historical truncation for domains over 255 bytes.
pub fn hash_with_domain(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update([domain.len() as u8]);
    hash.update(domain);
    hash.update(bytes);
    hash.finalize().into()
}

/// Extracts a 96-byte BLS key only from the exact network DER framing.
pub fn extract_raw_root_pk_from_der(der: &[u8]) -> Result<Vec<u8>, String> {
    if der.len() != 133 {
        return Err("invalid root pk length".into());
    }
    if !der.starts_with(IC_ROOT_PK_DER_PREFIX) {
        return Err("invalid OID".into());
    }
    Ok(der[IC_ROOT_PK_DER_PREFIX.len()..].to_vec())
}

#[cfg(test)]
mod tests;
