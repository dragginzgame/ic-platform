//! Passive consent-message content and response metadata.

use super::requests::ConsentMessageMetadata;
use serde::{Deserialize, Serialize};

/// A typed value in a field-oriented consent message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    /// A token quantity and its presentation information.
    TokenAmount {
        /// The decimal-place count supplied with the amount.
        decimals: u8,
        /// The raw token amount.
        amount: u64,
        /// The token symbol preserved verbatim.
        symbol: String,
    },
    /// A timestamp expressed in seconds.
    TimestampSeconds {
        /// The supplied timestamp value.
        amount: u64,
    },
    /// A duration expressed in seconds.
    DurationSeconds {
        /// The supplied duration value.
        amount: u64,
    },
    /// Uninterpreted UTF-8 text.
    Text {
        /// The text content.
        content: String,
    },
}

/// An intent description and ordered named display fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FieldsDisplay {
    /// The supplied description of the intended action.
    pub intent: String,
    /// Field labels and values in their supplied order.
    pub fields: Vec<(String, Value)>,
}

/// The content of a consent message, without granting authorization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentMessage {
    /// A general-purpose display string.
    GenericDisplayMessage(String),
    /// A structured field-oriented display message.
    FieldsDisplayMessage(FieldsDisplay),
}

/// A consent message and the metadata accompanying it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentInfo {
    /// The supplied presentation content.
    pub consent_message: ConsentMessage,
    /// The response's language and time-offset metadata.
    pub metadata: ConsentMessageMetadata,
}
