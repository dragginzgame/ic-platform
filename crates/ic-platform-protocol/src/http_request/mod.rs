//! Narrow management `http_request` encoding, without transport or HTTP policy.
//!
//! This capability supports GET with no headers or transform and unspecified
//! pricing version. URL bytes are preserved, not parsed, normalized or authorized.

/// Failure to construct the entire Candid request argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// The complete message exceeds the inclusive output allowance.
    OutputLimit {
        /// Caller-selected maximum encoded bytes.
        max_bytes: usize,
    },
    /// Output storage could not be reserved.
    Allocation,
}
impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutputLimit { max_bytes } => write!(f, "HTTP argument exceeds {max_bytes} bytes"),
            Self::Allocation => f.write_str("HTTP argument allocation failed"),
        }
    }
}
impl std::error::Error for EncodeError {}

// Ten Candid table entries: blob, opt blob, opt nat64, variant {get:null},
// header record, vec header, opt empty, opt bool, opt nat32, request record.
// The request's fields are ordered by Candid field hash, not source spelling.
// A GET-only variant is a subtype of the management method variant; opt empty
// represents the absent transform without importing a function/Principal type.
const HEADER: &[u8] = b"DIDL\x0a\x6d\x7b\x6e\x00\x6e\x78\x6b\x01\x96\x81\xba\x02\x7f\x6c\x02\xf1\xfe\xe1\x8d\x03\x71\xcb\xe4\xfd\xc7\x04\x71\x6d\x04\x6e\x6f\x6e\x7e\x6e\x79\x6c\x08\xef\xd6\xe4\x02\x71\xe1\xed\xeb\x4a\x03\xe8\xd6\xd8\x93\x01\x02\xa2\xf5\xed\x88\x04\x01\xec\xda\xcc\xac\x04\x06\xc6\xa4\xa1\x98\x06\x05\x90\xf8\xf6\xfc\x09\x07\x9f\xc5\x93\xca\x0f\x08\x01\x09";

/// Encodes exactly one management HTTP GET request argument.
///
/// Sets `max_response_bytes = Some(response_bytes)` and
/// `is_replicated = Some(replicated)`. Headers are empty; transform and pricing
/// version are absent. `None` and `Some(&[])` bodies remain distinct. URL and body
/// bytes are copied verbatim. No network request, cycle pricing, URL validation,
/// scheduling or response decoding occurs. Callers own those policies.
///
/// The complete length (including Candid tables) is checked before allocation.
/// An exact-size allowance succeeds; refusal never returns partial output. This
/// bounds the returned encoding, not process heap usage or the HTTP response.
///
/// # Errors
/// Returns [`EncodeError::OutputLimit`] on size overflow or an exceeded allowance,
/// and [`EncodeError::Allocation`] if reservation fails.
pub fn encode_http_get_args(
    url: &str,
    body: Option<&[u8]>,
    response_bytes: u64,
    replicated: bool,
    max_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    let refused = EncodeError::OutputLimit { max_bytes };
    // method index, Some(response limit)+nat64, body tag, transform tag,
    // header count, Some(replication)+bool, pricing tag = 16 bytes.
    let mut size = HEADER.len().checked_add(16).ok_or(refused)?;
    for amount in [leb_size(url.len()), url.len()] {
        size = size.checked_add(amount).ok_or(refused)?;
    }
    if let Some(body) = body {
        size = size
            .checked_add(leb_size(body.len()))
            .and_then(|n| n.checked_add(body.len()))
            .ok_or(refused)?;
    }
    if size > max_bytes {
        return Err(refused);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| EncodeError::Allocation)?;
    output.extend_from_slice(HEADER);
    leb(url.len(), &mut output);
    output.extend_from_slice(url.as_bytes());
    output.extend_from_slice(&[0, 1]); // GET, Some(max_response_bytes).
    output.extend_from_slice(&response_bytes.to_le_bytes());
    output.push(u8::from(body.is_some()));
    if let Some(body) = body {
        leb(body.len(), &mut output);
        output.extend_from_slice(body);
    }
    output.extend_from_slice(&[0, 0, 1, u8::from(replicated), 0]);
    debug_assert_eq!(size, output.len());
    Ok(output)
}

fn leb_size(mut value: usize) -> usize {
    let mut n = 1;
    while value >= 128 {
        n += 1;
        value >>= 7;
    }
    n
}
fn leb(mut value: usize, output: &mut Vec<u8>) {
    loop {
        let byte = (value & 127) as u8;
        value >>= 7;
        output.push(if value == 0 { byte } else { byte | 128 });
        if value == 0 {
            break;
        }
    }
}
