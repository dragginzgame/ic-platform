use super::super::icrc21::{
    ConsentMessageMetadata, ConsentMessageRequest, ConsentMessageSpec, DisplayMessageType,
};
crate::candid_record!(ConsentMessageMetadata {1502369582 => utc_offset_minutes: Option<i16>, 2047967320 => language: String});
crate::candid_variant!(DisplayMessageType {1750070539 => GenericDisplay, 2715993257 => FieldsDisplay});
crate::candid_record!(ConsentMessageSpec {1075439471 => metadata: ConsentMessageMetadata, 1534901700 => device_spec: Option<DisplayMessageType>});
crate::candid_record!(ConsentMessageRequest {4849238 => arg: Vec<u8>, 156956385 => method: String, 2904537988 => user_preferences: ConsentMessageSpec});
