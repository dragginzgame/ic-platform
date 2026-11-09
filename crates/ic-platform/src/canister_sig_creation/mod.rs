//! Upstream canister-signature key formats and hashing with owned issuer bytes.
//!
//! Encoding, decoding, hashing and domain separation delegate upstream. The
//! adapter preserves Platform Principal fields and checked DER framing. The
//! upstream signature store is not exported; consumers retain store lifecycle.

pub(crate) mod admission;

use crate::principal::Principal;

pub use ic_canister_sig_creation::{
    IC_ROOT_PK_DER, IC_ROOT_PK_DER_PREFIX, extract_raw_root_pk_from_der, hash_bytes,
    hash_with_domain,
};

/// An upstream canister-signature key with Platform's canonical issuer identity.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CanisterSigPublicKey {
    /// Canister issuing this signature key.
    pub canister_id: Principal,
    /// Consumer-selected seed, preserved without normalization.
    pub seed: Vec<u8>,
}

impl CanisterSigPublicKey {
    /// Construct a key without changing the issuer or seed.
    pub fn new(canister_id: Principal, seed: Vec<u8>) -> Self {
        Self { canister_id, seed }
    }

    fn from_upstream(key: ic_canister_sig_creation::CanisterSigPublicKey) -> Self {
        Self {
            canister_id: Principal::from_slice(key.canister_id.as_slice()),
            seed: key.seed,
        }
    }

    fn upstream(&self) -> ic_canister_sig_creation::CanisterSigPublicKey {
        ic_canister_sig_creation::CanisterSigPublicKey::new(
            self.canister_id
                .as_slice()
                .try_into()
                .expect("owned Principal has at most 29 bytes"),
            self.seed.clone(),
        )
    }

    /// Decode the upstream raw key format.
    ///
    /// # Errors
    /// Returns the unchanged upstream parsing diagnostic.
    pub fn try_from_raw(bytes: &[u8]) -> Result<Self, String> {
        ic_canister_sig_creation::CanisterSigPublicKey::try_from_raw(bytes).map(Self::from_upstream)
    }

    /// Encode the key using the upstream DER implementation.
    pub fn to_der(&self) -> Vec<u8> {
        self.upstream().to_der()
    }

    /// Encode the key using the upstream raw implementation.
    pub fn to_raw(&self) -> Vec<u8> {
        self.upstream().to_raw()
    }
}

impl TryFrom<&[u8]> for CanisterSigPublicKey {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        admission::canister_sig_der(bytes)?;
        ic_canister_sig_creation::CanisterSigPublicKey::try_from(bytes).map(Self::from_upstream)
    }
}
