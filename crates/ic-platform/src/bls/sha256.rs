//! Digest 0.10 trait compatibility; all hash computation belongs to SHA-2 0.11.

use digest_10::{
    FixedOutput, FixedOutputReset, HashMarker, Output, OutputSizeUser, Reset, Update,
    consts::{U32, U64},
    crypto_common::BlockSizeUser,
};
use sha2_11::Digest;

#[derive(Clone, Default)]
pub(crate) struct Sha256(sha2_11::Sha256);

impl OutputSizeUser for Sha256 {
    type OutputSize = U32;
}

impl BlockSizeUser for Sha256 {
    type BlockSize = U64;
}

impl HashMarker for Sha256 {}

impl Update for Sha256 {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

impl FixedOutput for Sha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(&self.0.finalize());
    }
}

impl Reset for Sha256 {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl FixedOutputReset for Sha256 {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        out.copy_from_slice(&self.0.finalize_reset());
    }
}
