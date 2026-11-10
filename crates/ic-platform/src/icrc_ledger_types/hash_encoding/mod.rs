//! Unsigned LEB128 payload encoding for ledger natural numbers.
//!
//! This module owns only integer bytes, not hashing or domain selection.

use crate::candid_codec::Nat;
use std::io::{self, Write};

impl Nat {
    /// Appends the shortest unsigned LEB128 payload to `writer`.
    ///
    /// Write failures propagate and may leave a prefix of the payload appended.
    /// The writer is never flushed.
    pub(crate) fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        let mut digits = self.0.iter_u32_digits();
        let mut remaining_bits = self.0.bits();
        let mut pending = 0_u64;
        let mut pending_bits = 0_u32;
        let mut output = [0_u8; 64];
        let mut filled = 0;

        loop {
            // At most six bits remain before loading a 32-bit digit, so the
            // pending value fits in u64. A final group may use fewer than seven.
            if pending_bits < 7
                && let Some(digit) = digits.next()
            {
                pending |= u64::from(digit) << pending_bits;
                pending_bits += 32;
            }

            let mut byte = (pending & 0x7f) as u8;
            pending >>= 7;
            pending_bits = pending_bits.saturating_sub(7);
            remaining_bits = remaining_bits.saturating_sub(7);
            if remaining_bits != 0 {
                byte |= 0x80;
            }
            output[filled] = byte;
            filled += 1;

            // The loop emits one byte even for zero. Magnitude bits, rather
            // than the padded width of the last digit, determine termination.
            if remaining_bits == 0 {
                return writer.write_all(&output[..filled]);
            }
            if filled == output.len() {
                writer.write_all(&output)?;
                filled = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests;
