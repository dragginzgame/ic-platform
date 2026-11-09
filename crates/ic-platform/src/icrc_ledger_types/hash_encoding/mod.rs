//! Upstream Candid natural encoding for unchanged ICRC hash inputs.
//! See ../NOTICE.md; the source guard protects this imported implementation.
use crate::candid_codec::Nat;
use std::io;
#[cfg(test)]
mod tests;
impl Nat {
    pub(crate) fn encode<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        use num_traits::cast::ToPrimitive;
        if let Some(value) = self.0.to_u64() {
            leb128::write::unsigned(w, value)?;
            return Ok(());
        }
        // Large value: emit the base-128 LEB128 groups in a single O(n) pass.
        // Radix 128 is a power of two, so `to_radix_le` bit-unpacks in O(n)
        // instead of the O(n^2) shift-the-whole-bignum-by-7-each-byte loop.
        let mut groups = self.0.to_radix_le(128);
        let last = groups.len() - 1;
        for byte in &mut groups[..last] {
            *byte |= 0x80u8; // continuation bit on every group but the last
        }
        w.write_all(&groups)?;
        Ok(())
    }
}
