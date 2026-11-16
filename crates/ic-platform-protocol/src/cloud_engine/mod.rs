//! Selected Cloud Engine wire formats.
//!
//! The caller owns endpoint selection, identity, deadlines, request-count policy
//! and interpretation. Encoding neither sends a request nor authenticates data.

use std::fmt;

/// A refusal to construct the complete request frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// The complete frame exceeds the caller's byte allowance.
    OutputLimit {
        /// Maximum permitted encoded bytes, including the header.
        max_bytes: usize,
    },
    /// The allocator refused storage for an admitted frame.
    Allocation,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutputLimit { max_bytes } => {
                write!(
                    formatter,
                    "request exceeds the {max_bytes}-byte output limit"
                )
            }
            Self::Allocation => formatter.write_str("request allocation failed"),
        }
    }
}

impl std::error::Error for EncodeError {}

/// Encode one `getEngineViews` argument: a vector of opaque UTF-8 engine IDs.
///
/// An empty slice is one empty vector, not zero arguments. Text is preserved
/// byte-for-byte, including empty strings, NUL and whitespace. The caller owns
/// service-specific count and ID validation. The complete encoded length is
/// checked before allocating output; this bound is not a process heap quota.
///
/// # Errors
/// Returns [`EncodeError::OutputLimit`] if the header and payload exceed
/// `max_bytes`, or [`EncodeError::Allocation`] if output allocation fails.
pub fn encode_get_engine_views_args(
    ids: &[String],
    max_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    // DIDL, one table entry (vec text), one argument referring to entry zero.
    const HEADER: &[u8] = b"DIDL\x01\x6d\x71\x01\x00";
    let refusal = EncodeError::OutputLimit { max_bytes };
    let mut size = HEADER.len();
    let mut charge = |amount: usize| -> Result<(), EncodeError> {
        size = size.checked_add(amount).ok_or(refusal)?;
        if size > max_bytes {
            return Err(refusal);
        }
        Ok(())
    };
    charge(uleb_size(ids.len()))?;
    for id in ids {
        charge(uleb_size(id.len()))?;
        charge(id.len())?;
    }

    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| EncodeError::Allocation)?;
    output.extend_from_slice(HEADER);
    write_uleb(ids.len(), &mut output);
    for id in ids {
        write_uleb(id.len(), &mut output);
        output.extend_from_slice(id.as_bytes());
    }
    Ok(output)
}

fn uleb_size(mut value: usize) -> usize {
    let mut size = 1;
    while value >= 128 {
        value >>= 7;
        size += 1;
    }
    size
}

fn write_uleb(mut value: usize, output: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        output.push(if value == 0 { byte } else { byte | 0x80 });
        if value == 0 {
            return;
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(feature = "cloud-engine-response")]
mod response;
#[cfg(feature = "cloud-engine-response")]
pub use response::{
    EngineSubnet, EngineView, RegistryCopyStatus, decode_get_engine_views_response,
    decode_get_network_fee_response, decode_registry_copy_status_response,
};

#[cfg(feature = "cloud-engine-marketplace")]
pub use response::{
    AvailableNode, AvailableNodesPage, AvailableNodesPageResult, EngineError,
    ListAvailableNodesPageArgs, MarketplaceEntry, NodeType, Price, PriceEntry,
    decode_list_available_nodes_page_response, decode_list_marketplace_prices_response,
    encode_list_available_nodes_page_args,
};

/// Encode the external service's zero-argument Candid frame.
///
/// # Errors
/// Refuses an output allowance smaller than six bytes.
pub fn encode_empty_args(max_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    if max_bytes < 6 {
        return Err(EncodeError::OutputLimit { max_bytes });
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(6)
        .map_err(|_| EncodeError::Allocation)?;
    bytes.extend_from_slice(b"DIDL\0\0");
    Ok(bytes)
}
