//! Delegation to the registry owner of self-authenticating Principal derivation.

use super::Principal;

impl Principal {
    /// Derive an identity through the upstream Principal implementation.
    ///
    /// The caller supplies the public key bytes. This adapter preserves the
    /// returned identity bytes and adds no parsing or authorization policy.
    pub fn self_authenticating<P: AsRef<[u8]>>(public_key: P) -> Self {
        Self::from_slice(ic_principal::Principal::self_authenticating(public_key).as_slice())
    }
}

#[cfg(test)]
mod tests;
