//! Passive external ICRC-21 consent DTOs; consumers own display and consent policy.

mod errors;
mod requests;
mod responses;

pub use errors::{ErrorInfo, Icrc21Error};
pub use requests::{
    ConsentMessageMetadata, ConsentMessageRequest, ConsentMessageSpec, DisplayMessageType,
};
pub use responses::{ConsentInfo, ConsentMessage, FieldsDisplay, Value};
