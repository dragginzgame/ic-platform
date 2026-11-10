//! Consent-message requests and caller display preferences.

use serde::{Deserialize, Serialize};

/// Language and time-offset preferences accompanying a consent message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentMessageMetadata {
    /// The caller's language string, without local validation.
    pub language: String,
    /// An optional UTC offset in minutes.
    pub utc_offset_minutes: Option<i16>,
}

/// The caller's preferred message presentation shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMessageType {
    /// A single general-purpose display message.
    GenericDisplay,
    /// A message expressed as named fields.
    FieldsDisplay,
}

/// Metadata and an optional device presentation preference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentMessageSpec {
    /// The caller's language and time preferences.
    pub metadata: ConsentMessageMetadata,
    /// The optional requested display shape.
    pub device_spec: Option<DisplayMessageType>,
}

/// A request to describe a canister call for consent presentation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentMessageRequest {
    /// The target method name preserved verbatim.
    pub method: String,
    /// Raw argument bytes using ordinary sequence serialization.
    pub arg: Vec<u8>,
    /// The requested message and device preferences.
    pub user_preferences: ConsentMessageSpec,
}
