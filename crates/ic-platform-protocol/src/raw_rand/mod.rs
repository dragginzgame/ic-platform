//! Management `raw_rand`'s bounded canonical producer profile.
//!
//! Admits one Candid type-table entry (`vec nat8`), one argument referring to that
//! entry, and exactly 32 bytes with canonical single-byte lengths/type references.
//! This intentionally rejects alternate equivalent Candid encodings. The selected
//! producer profile has exactly 42 bytes; work and storage are fixed, with no heap
//! allocation. Zero seed bytes are valid protocol data. Readiness, entropy mixing,
//! transport, cycles, stale-result admission and retries remain consumer-owned.

/// Management method name; no call is issued by this module.
pub const METHOD: &str = "raw_rand";
/// Canonical empty Candid argument tuple.
const ARGUMENTS: &[u8] = b"DIDL\0\0";
/// Canonical selected producer header, excluding the vector length.
const HEADER: &[u8] = b"DIDL\x01\x6d\x7b\x01\x00";

/// Refusal without partial seed output or allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Caller-selected complete byte allowance was exceeded.
    ByteLimit,
    /// Framing, type table or arity does not match the selected producer profile.
    Framing,
    /// The complete reply does not contain exactly the required 32-byte vector.
    Length,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw_rand protocol refusal: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Borrows the exact empty argument frame after checking the complete byte bound.
///
/// # Errors
/// Refuses an allowance below six bytes. This operation never allocates.
pub fn encode(maximum_bytes: usize) -> Result<&'static [u8], Error> {
    if maximum_bytes < ARGUMENTS.len() {
        Err(Error::ByteLimit)
    } else {
        Ok(ARGUMENTS)
    }
}

/// Decodes one canonical 32-byte management response into owned fixed storage.
/// The whole-input ceiling is checked before framing; no skipped data is accepted.
///
/// # Errors
/// Refuses over-limit, truncated, trailing, wrong-length or non-profile framing.
pub fn decode(bytes: &[u8], maximum_bytes: usize) -> Result<[u8; 32], Error> {
    if bytes.len() > maximum_bytes {
        return Err(Error::ByteLimit);
    }
    if bytes.len() != 42 {
        return Err(Error::Length);
    }
    if !bytes.starts_with(HEADER) {
        return Err(Error::Framing);
    }
    if bytes[9] != 32 {
        return Err(Error::Length);
    }
    let mut seed = [0; 32];
    seed.copy_from_slice(&bytes[10..]);
    Ok(seed)
}

#[cfg(test)]
mod tests;
