//! Size probes for reachable Platform capabilities, not application benchmarks.

fn input() -> Vec<u8> {
    let mut bytes = vec![0; ic_platform::ic0::msg_arg_data_size()];
    ic_platform::ic0::msg_arg_data_copy(&mut bytes, 0);
    bytes
}

fn reply(bytes: &[u8]) {
    ic_platform::ic0::msg_reply_data_append(bytes);
    ic_platform::ic0::msg_reply();
}

#[cfg(feature = "principal")]
mod principal {
    use super::*;
    use ic_platform::{Account, Principal};

    fn finish(result: Result<Principal, ic_platform::PrincipalError>) {
        match result {
            Ok(owner) => {
                let account = Account {
                    owner,
                    subaccount: Some([42; 32]),
                };
                let json = serde_json::to_vec(&account).unwrap();
                let text = account.to_string();
                reply(&json);
                // Keep Account text formatting independently reachable.
                ic_platform::ic0::debug_print(text.as_bytes());
            }
            Err(error) => reply(error.to_string().as_bytes()),
        }
    }

    #[unsafe(export_name = "canister_query parse_slice")]
    pub extern "C" fn parse_slice() {
        let bytes = input();
        finish(Principal::from_text(std::str::from_utf8(&bytes).unwrap()));
    }

    #[unsafe(export_name = "canister_query parse_string")]
    pub extern "C" fn parse_string() {
        finish(Principal::from_text(String::from_utf8(input()).unwrap()));
    }

    #[unsafe(export_name = "canister_query parse_string_ref")]
    pub extern "C" fn parse_string_ref() {
        let text = String::from_utf8(input()).unwrap();
        finish(Principal::from_text(&text));
    }
}

#[cfg(feature = "runtime")]
#[unsafe(export_name = "canister_update raw_call")]
pub extern "C" fn raw_call() {
    use ic_platform::cdk::{api, call::Call, futures};
    futures::internals::in_executor_context(|| {
        futures::spawn(async {
            let bytes = api::msg_arg_data();
            match Call::bounded_wait(api::canister_self(), "probe")
                .take_raw_args(bytes)
                .await
            {
                Ok(response) => api::msg_reply(response),
                Err(error) => api::msg_reject(error.to_string()),
            }
        });
    });
}

#[cfg(feature = "external-candid")]
#[unsafe(export_name = "canister_query external_status")]
pub extern "C" fn external_status() {
    use ic_platform::{candid, management_canister_types::CanisterStatusResult};
    let bytes = input();
    let limits = candid::DecodeLimits {
        reply_bytes: 1_000_000,
        decoding_quota: 1_000_000,
        skipping_quota: 1_000_000,
        type_entries: 1_000,
        header_bytes: 100_000,
    };
    let result = candid::decode_exactly_one::<CanisterStatusResult>(&bytes, limits);
    match result {
        Ok(status) => reply(&candid::contract::encode_one(status).unwrap()),
        Err(error) => reply(error.to_string().as_bytes()),
    }
}

#[cfg(feature = "verification")]
#[unsafe(export_name = "canister_query verify")]
pub extern "C" fn verify() {
    let bytes = input();
    let quarter = bytes.len() / 4;
    let (message, rest) = bytes.split_at(quarter);
    let (signature, rest) = rest.split_at(quarter);
    let (key, root) = rest.split_at(quarter);
    let result =
        ic_platform::signature_verification::verify_canister_sig(message, signature, key, root);
    reply(&[u8::from(result.is_ok())]);
}

#[cfg(feature = "principal-binary")]
#[unsafe(export_name = "canister_query binary_principal")]
pub extern "C" fn binary_principal() {
    use ic_platform::{Account, Principal};
    match Principal::try_from_slice(&input()) {
        Ok(owner) => reply(Account::from(owner).owner.as_slice()),
        Err(_) => reply(&[]),
    }
}

#[cfg(feature = "principal-self-auth")]
#[unsafe(export_name = "canister_query self_auth_slice")]
pub extern "C" fn self_auth_slice() {
    let bytes = input();
    reply(ic_platform::Principal::self_authenticating(bytes.as_slice()).as_slice());
}

#[cfg(feature = "principal-self-auth")]
#[unsafe(export_name = "canister_query self_auth_vec")]
pub extern "C" fn self_auth_owned() {
    reply(ic_platform::Principal::self_authenticating(input()).as_slice());
}

#[cfg(feature = "principal-self-auth")]
#[unsafe(export_name = "canister_query self_auth_vec_ref")]
pub extern "C" fn self_auth_borrowed() {
    let bytes = input();
    reply(ic_platform::Principal::self_authenticating(&bytes).as_slice());
}

#[cfg(any(feature = "candid-exact", feature = "candid-sequence"))]
fn candid_size_limits() -> ic_platform::candid::DecodeLimits {
    ic_platform::candid::DecodeLimits {
        reply_bytes: 1_000_000,
        decoding_quota: 1_000_000,
        skipping_quota: 0,
        type_entries: 1_000,
        header_bytes: 100_000,
    }
}

// Dedicated external management DTO qualification, not owned endpoint schemas.
#[cfg(feature = "candid-exact")]
fn exact_reply<
    T: ic_platform::candid::contract::CandidType
        + for<'de> ic_platform::candid::contract::serde::Deserialize<'de>,
>() {
    let wire = input();
    match ic_platform::candid::decode_exactly_one::<T>(&wire, candid_size_limits()) {
        Ok(value) => reply(&ic_platform::candid::contract::encode_one(value).unwrap()),
        Err(error) => reply(error.to_string().as_bytes()),
    }
}

#[cfg(feature = "candid-exact")]
macro_rules! external_exact_probe {
    ($name:ident, $export:literal, $dto:ident) => {
        #[unsafe(export_name = $export)]
        pub extern "C" fn $name() {
            exact_reply::<ic_platform::management_canister_types::$dto>();
        }
    };
}

#[cfg(feature = "candid-exact")]
external_exact_probe!(
    exact_status,
    "canister_query exact_status",
    CanisterStatusResult
);
#[cfg(feature = "candid-exact")]
external_exact_probe!(exact_http, "canister_query exact_http", HttpRequestResult);
#[cfg(feature = "candid-exact")]
external_exact_probe!(exact_key, "canister_query exact_key", EcdsaPublicKeyResult);
#[cfg(feature = "candid-exact")]
external_exact_probe!(
    exact_signature,
    "canister_query exact_signature",
    SignWithEcdsaResult
);

// A synthetic external vec nat8 contract exercises caller-selected limits.
#[cfg(feature = "candid-sequence")]
struct ExternalBoundedBytes<const LIMIT: usize>(Vec<u8>);

#[cfg(feature = "candid-sequence")]
impl<const LIMIT: usize> ic_platform::candid::contract::CandidType for ExternalBoundedBytes<LIMIT> {
    fn _ty() -> ic_platform::candid::contract::types::Type {
        <Vec<u8> as ic_platform::candid::contract::CandidType>::_ty()
    }

    fn idl_serialize<S: ic_platform::candid::contract::types::Serializer>(
        &self,
        serializer: S,
    ) -> Result<(), S::Error> {
        ic_platform::candid::contract::CandidType::idl_serialize(&self.0, serializer)
    }
}

#[cfg(feature = "candid-sequence")]
impl<'de, const LIMIT: usize> ic_platform::candid::contract::serde::Deserialize<'de>
    for ExternalBoundedBytes<LIMIT>
{
    fn deserialize<D: ic_platform::candid::contract::serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        ic_platform::candid::sequence::deserialize::<D, u8, LIMIT>(
            deserializer,
            "bounded external bytes",
            "external byte limit",
        )
        .map(Self)
    }
}

#[cfg(feature = "candid-sequence")]
fn sequence_reply<const LIMIT: usize>() {
    let wire = input();
    match ic_platform::candid::decode_exactly_one::<ExternalBoundedBytes<LIMIT>>(
        &wire,
        candid_size_limits(),
    ) {
        Ok(value) => reply(&value.0),
        Err(error) => reply(error.to_string().as_bytes()),
    }
}

#[cfg(feature = "candid-sequence")]
macro_rules! external_sequence_probe {
    ($name:ident, $export:literal, $limit:literal) => {
        #[unsafe(export_name = $export)]
        pub extern "C" fn $name() {
            sequence_reply::<$limit>();
        }
    };
}

#[cfg(feature = "candid-sequence")]
#[cfg(not(feature = "candid-sequence-one"))]
external_sequence_probe!(sequence_zero, "canister_query sequence_zero", 0);
#[cfg(feature = "candid-sequence")]
external_sequence_probe!(sequence_two, "canister_query sequence_two", 2);
#[cfg(feature = "candid-sequence")]
#[cfg(not(feature = "candid-sequence-one"))]
external_sequence_probe!(sequence_sixteen, "canister_query sequence_sixteen", 16);
#[cfg(feature = "candid-sequence")]
#[cfg(not(feature = "candid-sequence-one"))]
external_sequence_probe!(sequence_256, "canister_query sequence_256", 256);
