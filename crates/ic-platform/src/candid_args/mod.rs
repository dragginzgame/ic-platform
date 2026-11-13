//! Small external Candid argument encoders without the general Candid library.
//!
//! Only empty tuples, lists of unit/null arguments and a single text argument are
//! supported. Callers retain method selection, budgets and transport. Owned
//! application protocols continue to use Wire.

/// Exact zero-argument envelope, without allocation or failure.
#[must_use]
pub const fn empty() -> &'static [u8; 6] {
    b"DIDL\0\0"
}

/// Encode `count` unit/null arguments: zero gives `()`, two gives `((), ())`.
///
/// The count is caller-selected; no resource limit is inferred.
#[must_use]
pub fn units(count: usize) -> Vec<u8> {
    let mut bytes = header(count, count);
    bytes.extend(core::iter::repeat_n(0x7f, count));
    bytes
}

/// Encode one text argument with its exact UTF-8 bytes and byte length.
#[must_use]
pub fn text(value: &str) -> Vec<u8> {
    let mut bytes = header(1, 1 + unsigned_len(value.len()) + value.len());
    bytes.push(0x71);
    unsigned(value.len(), &mut bytes);
    bytes.extend_from_slice(value.as_bytes());
    bytes
}

fn header(count: usize, payload_bytes: usize) -> Vec<u8> {
    // Saturation leaves impossible sizes to Vec's ordinary capacity refusal.
    let capacity = (5 + unsigned_len(count)).saturating_add(payload_bytes);
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend_from_slice(b"DIDL\0");
    unsigned(count, &mut bytes);
    bytes
}

fn unsigned_len(value: usize) -> usize {
    ((usize::BITS - value.leading_zeros()).max(1) as usize).div_ceil(7)
}

fn unsigned(mut value: usize, bytes: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        bytes.push(byte | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            return;
        }
    }
}
