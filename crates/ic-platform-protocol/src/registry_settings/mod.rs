//! Selected Registry subnet settings, preserving raw protobuf presence.
//!
//! This allocation-free projection authenticates nothing and resolves no replica
//! defaults. It consumes an already extracted SubnetRecord, not a Registry envelope.

#[cfg(test)]
mod tests;
pub(crate) mod wire;

use serde::{Deserialize, Serialize};
use wire::{FieldValue, Read, Reader};

/// Mandatory limits shared by the outer record and all selected nested messages.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum complete SubnetRecord bytes, including unknown fields.
    pub bytes: usize,
    /// Maximum total field occurrences, including unknown and nested fields.
    pub fields: usize,
}

/// Refusals from this selected protobuf projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Complete input exceeds the caller's byte allowance.
    ByteLimit,
    /// The shared field allowance is exhausted.
    FieldLimit,
    /// Truncated, overflowing, mistyped or otherwise invalid input.
    Malformed,
    /// Groups or unknown protobuf wire types are unsupported.
    UnsupportedWireType,
    /// A selected singular field occurs more than once, including default values.
    DuplicateField,
}

// Each declaration owns its field numbers, Rust types, defaults and dispatch.
// Only the fixed, two-level selected message graph is reachable; no generic
// schema reflection, encoder or caller-defined recursive mapping is exposed.
macro_rules! message {
    ($(#[$meta:meta])* $name:ident { $($(#[$doc:meta])* $number:literal => $field:ident : $ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
        pub struct $name { $($(#[$doc])* pub $field: $ty,)+ }
        const _: () = {
            let labels = [$($number),+];
            let mut index = 0;
            while index < labels.len() {
                assert!(labels[index] > 0 && labels[index] < 64);
                if index > 0 { assert!(labels[index - 1] < labels[index]); }
                index += 1;
            }
        };
        impl Read for $name {
            fn read(value: FieldValue<'_>, remaining: &mut usize) -> Result<Self, Error> {
                let FieldValue::Bytes(bytes) = value else { return Err(Error::Malformed); };
                let mut reader = Reader::new(bytes);
                let mut result = Self::default();
                let mut seen = 0u64;
                while let Some((number, value)) = reader.next(remaining)? {
                    match number {
                        $($number => {
                            let bit = 1u64 << $number;
                            if seen & bit != 0 { return Err(Error::DuplicateField); }
                            seen |= bit;
                            result.$field = <$ty as Read>::read(value, remaining)?;
                        },)+
                        _ => (),
                    }
                }
                Ok(result)
            }
        }
    };
}

message! {
    ///
    /// SubnetFeatures
    /// Raw configured features; no execution or attestation claim is implied.
    ///

    SubnetFeatures {
        /// Proto3 default is false when absent within a present message.
        3 => http_requests: bool,
        /// Configured SEV presence/value, not verified hardware attestation.
        9 => sev_enabled: Option<bool>,
    }
}

message! {
    ///
    /// ResourceLimits
    /// Raw optional limits; absent and zero are retained independently.
    ///

    ResourceLimits {
        /// Configured state-size bytes.
        1 => maximum_state_size: Option<u64>,
        /// Configured state-delta bytes.
        2 => maximum_state_delta: Option<u64>,
        /// Configured query instructions; zero selects a replica default.
        3 => maximum_query_instructions: Option<u64>,
        /// Configured query wall time in seconds; zero selects a replica default.
        4 => maximum_query_walltime_seconds: Option<u64>,
    }
}

message! {
    ///
    /// SubnetSettings
    /// Selected SubnetRecord settings, without transport or effective-value policy.
    ///

    SubnetSettings {
        /// Absent differs from a present empty feature message.
        23 => features: Option<SubnetFeatures>,
        /// Raw canister-count setting; absent defaults to zero (replica default).
        24 => max_number_of_canisters: u64,
        /// Absent differs from a present empty limits message.
        34 => resource_limits: Option<ResourceLimits>,
    }
}

/// Decode selected settings from an already extracted Registry SubnetRecord.
///
/// No heap storage is allocated. Unknown fields are skipped after validating
/// their wire framing; opaque length-delimited bytes are not interpreted as
/// nested messages. Selected nested messages share the same field allowance.
/// Field order is unrestricted; duplicate selected singular fields refuse.
///
/// # Errors
/// Returns a typed byte/field limit, malformed-data, duplicate-field or unsupported
/// wire-type refusal. It does not verify a Registry envelope, identity or signature.
pub fn decode_subnet_settings(bytes: &[u8], limits: Limits) -> Result<SubnetSettings, Error> {
    if bytes.len() > limits.bytes {
        return Err(Error::ByteLimit);
    }
    let mut remaining = limits.fields;
    SubnetSettings::read(FieldValue::Bytes(bytes), &mut remaining)
}
