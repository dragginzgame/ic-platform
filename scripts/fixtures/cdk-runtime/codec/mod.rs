//! Caller-owned Wire adapters; Platform only owns entrypoint/callback contexts.
use crate::protocol::{LIMITS, method_id};
use ic_platform::cdk::api;
use serde::{Serialize, de::DeserializeOwned};

pub fn decode<T: Serialize + DeserializeOwned, const ID: u64>(bytes: Vec<u8>) -> T {
    ic_wire::native::decode_request(ID, &bytes, LIMITS)
        .unwrap_or_else(|error| api::trap(format!("fixture request: {error}")))
}
pub fn encode<T: Serialize, const ID: u64>(value: T) -> Vec<u8> {
    ic_wire::native::encode_response(ID, &value, LIMITS)
        .unwrap_or_else(|error| api::trap(format!("fixture response: {error}")))
}
pub fn encode_unit<const ID: u64>() -> Vec<u8> {
    encode::<_, ID>(())
}
pub fn request<T: Serialize>(name: &str, value: &T) -> Vec<u8> {
    ic_wire::native::encode_request(method_id(name), value, LIMITS).unwrap()
}
pub fn response<T: Serialize + DeserializeOwned>(name: &str, bytes: &[u8]) -> T {
    ic_wire::native::decode_response(method_id(name), bytes, LIMITS).unwrap()
}

pub fn admit() -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > LIMITS.bytes {
        return Err("fixture request exceeds Wire byte allowance".into());
    }
    Ok(())
}
