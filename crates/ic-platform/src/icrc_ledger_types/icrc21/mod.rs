//! External ICRC-21 consent DTOs with their original upstream identities.
//!
//! This explicit namespace keeps consent Value distinct from the ledger generic
//! Value. Consumers own consent policy, authentication and endpoint mounting.

pub use icrc_ledger_types::icrc21::errors::{ErrorInfo, Icrc21Error};
pub use icrc_ledger_types::icrc21::requests::{
    ConsentMessageMetadata, ConsentMessageRequest, ConsentMessageSpec, DisplayMessageType,
};
pub use icrc_ledger_types::icrc21::responses::{ConsentInfo, ConsentMessage, FieldsDisplay, Value};
