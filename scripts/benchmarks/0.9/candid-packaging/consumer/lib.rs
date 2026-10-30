#[cfg(not(feature = "owned-codec"))]
use candid as selected;
use candid::CandidType;
#[cfg(feature = "owned-codec")]
use codec as selected;
use ic_platform::candid::{DecodeError, DecodeLimits, nesting};
use ic_platform::management_canister_types::CanisterStatusResult;
use selected::de::{DecoderConfig, IDLDeserialize};

fn encode<T: CandidType>(value: T) -> candid::Result<Vec<u8>> {
    let mut builder = selected::ser::IDLBuilder::new();
    builder.arg(&value)?;
    builder.serialize_to_vec()
}

fn limits() -> DecodeLimits {
    DecodeLimits {
        reply_bytes: 1_000_000,
        decoding_quota: 1_000_000,
        skipping_quota: 1_000_000,
        type_entries: 10_000,
        header_bytes: 64 * 1024,
    }
}

fn decode(bytes: &[u8]) -> Result<CanisterStatusResult, DecodeError> {
    let limits = limits();
    if bytes.len() > limits.reply_bytes {
        return Err(DecodeError::ReplyTooLarge {
            actual: bytes.len(),
            maximum: limits.reply_bytes,
        });
    }
    nesting::check(bytes, limits)
        .map_err(|error| DecodeError::Candid(candid::Error::Custom(error.into())))?;
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(limits.decoding_quota)
        .set_skipping_quota(limits.skipping_quota)
        .set_max_type_len(limits.type_entries)
        .set_max_header_len(limits.header_bytes)
        .set_full_error_message(false);
    let mut decoder =
        IDLDeserialize::new_with_config(bytes, &config).map_err(DecodeError::Candid)?;
    if decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "expected exactly one Candid result",
        )));
    }
    let value = decoder.get_value().map_err(DecodeError::Candid)?;
    if !decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "expected exactly one Candid result",
        )));
    }
    decoder.done().map_err(DecodeError::Candid)?;
    Ok(value)
}

fn input(bytes: Vec<u8>) -> Result<CanisterStatusResult, String> {
    decode(&bytes).map_err(|e| e.to_string())
}
fn output(result: Result<CanisterStatusResult, String>) -> Vec<u8> {
    match result {
        Ok(value) => encode(value).unwrap(),
        Err(error) => error.into_bytes(),
    }
}
#[ic_platform::cdk::query(decode_with = "input", encode_with = "output")]
fn status(value: Result<CanisterStatusResult, String>) -> Result<CanisterStatusResult, String> {
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_dtos_use_the_selected_codec_without_conversions() {
        let hex: String = include_str!("fixtures/management-snapshot-status.hex")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let bytes: Vec<_> = hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        let statuses: Vec<CanisterStatusResult> = candid::decode_one(&bytes).unwrap();
        assert_eq!(statuses.len(), 3);
        for status in statuses {
            let wire = candid::encode_one(&status).unwrap();
            assert_eq!(encode(&status).unwrap(), wire);
            assert_eq!(decode(&wire).unwrap(), status);
        }
    }

    #[test]
    fn registry_error_identity_is_retained() {
        let error: candid::Error = IDLDeserialize::new(b"invalid").err().unwrap();
        assert!(!error.to_string().is_empty());
    }
}

#[cfg(feature = "mixed-codec")]
fn registry_input(bytes: Vec<u8>) -> Result<CanisterStatusResult, String> {
    ic_platform::candid::decode_exactly_one(&bytes, limits()).map_err(|error| error.to_string())
}
#[cfg(feature = "mixed-codec")]
fn registry_output(result: Result<CanisterStatusResult, String>) -> Vec<u8> {
    match result {
        Ok(value) => candid::encode_one(value).unwrap(),
        Err(error) => error.into_bytes(),
    }
}
#[cfg(feature = "mixed-codec")]
#[ic_platform::cdk::query(decode_with = "registry_input", encode_with = "registry_output")]
fn registry_status(
    value: Result<CanisterStatusResult, String>,
) -> Result<CanisterStatusResult, String> {
    value
}
