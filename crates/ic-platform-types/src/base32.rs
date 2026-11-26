//! Unpadded RFC 4648 Base32 for bounded identity text.

const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

pub(crate) fn encode(bytes: &[u8], output: &mut [u8]) -> usize {
    let mut buffer = 0_u16;
    let mut bits = 0;
    let mut written = 0;
    for &byte in bytes {
        buffer = (buffer << 8) | u16::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            output[written] = ALPHABET[usize::from((buffer >> bits) & 31)];
            written += 1;
        }
    }
    if bits != 0 {
        output[written] = ALPHABET[usize::from((buffer << (5 - bits)) & 31)];
        written += 1;
    }

    written
}

pub(crate) fn decode_dashed(text: &str, output: &mut [u8]) -> Option<usize> {
    let mut buffer = 0_u16;
    let mut bits = 0;
    let mut symbols = 0;
    let mut written = 0;
    for byte in text.bytes() {
        let byte = byte.to_ascii_lowercase();
        let digit = match byte {
            b'a'..=b'z' => byte - b'a',
            b'2'..=b'7' => byte - b'2' + 26,
            b'-' => continue,
            _ => return None,
        };
        symbols += 1;
        buffer = (buffer << 5) | u16::from(digit);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            *output.get_mut(written)? = (buffer >> bits) as u8;
            written += 1;
        }
    }

    // The last symbol may contain zero padding bits. Extra whole symbols
    // cannot represent an additional byte in the unpadded encoding.
    (symbols == (written * 8).div_ceil(5) && buffer & ((1 << bits) - 1) == 0).then_some(written)
}
