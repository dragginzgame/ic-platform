use super::super::icrc21::{
    ConsentInfo, ConsentMessage, ConsentMessageMetadata, FieldsDisplay, Value,
};
crate::candid_variant!(Value {936573133 => Text{427265337 => content: String}, 1289986449 => TokenAmount{308955842 => decimals: u8, 3573748184 => amount: u64, 4007505752 => symbol: String}, 2826488937 => TimestampSeconds{3573748184 => amount: u64}, 4208601451 => DurationSeconds{3573748184 => amount: u64}});
crate::candid_record!(FieldsDisplay {2156826233 => fields: Vec<(String, Value)>, 2659612252 => intent: String});
crate::candid_variant!(ConsentMessage {124612638 => FieldsDisplayMessage(FieldsDisplay), 4082495484 => GenericDisplayMessage(String)});
crate::candid_record!(ConsentInfo {1075439471 => metadata: ConsentMessageMetadata, 1763119074 => consent_message: ConsentMessage});
