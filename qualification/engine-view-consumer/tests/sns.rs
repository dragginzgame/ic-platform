//! Independent Candid schemas for the pinned SNS neuron and ICRC display boundary.
#![forbid(unsafe_code)]
#![cfg(all(feature = "sns-neurons", not(target_family = "wasm")))]
use candid::{CandidType, Nat, Principal};
use ic_platform_protocol::{
    candid::{Error, Limits},
    ledger, sns,
};
use serde::Deserialize;

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
struct NeuronId {
    id: Vec<u8>,
}
#[derive(CandidType, Deserialize, PartialEq, Debug)]
struct GetNeuron {
    neuron_id: Option<NeuronId>,
}
#[derive(CandidType, Deserialize, PartialEq, Debug)]
struct ListNeurons {
    limit: u32,
    of_principal: Option<Principal>,
    start_page_at: Option<NeuronId>,
}
#[derive(Clone, CandidType, Deserialize)]
struct Permission {
    principal: Option<Principal>,
    permission_type: Vec<i32>,
}
#[derive(Clone, CandidType, Deserialize)]
enum DissolveState {
    DissolveDelaySeconds(u64),
    WhenDissolvedTimestampSeconds(u64),
}
#[derive(Clone, CandidType, Deserialize)]
struct Followees {
    followees: Vec<NeuronId>,
}
#[derive(Clone, CandidType, Deserialize)]
struct RewardEventParticipation {
    reward_event_end_timestamp_seconds: Option<u64>,
    reward_shares: Option<Nat>,
}
#[derive(Clone, CandidType, Deserialize)]
struct Neuron {
    id: Option<NeuronId>,
    permissions: Vec<Permission>,
    cached_neuron_stake_e8s: u64,
    neuron_fees_e8s: u64,
    maturity_e8s_equivalent: u64,
    staked_maturity_e8s_equivalent: Option<u64>,
    created_timestamp_seconds: u64,
    aging_since_timestamp_seconds: u64,
    dissolve_state: Option<DissolveState>,
    auto_stake_maturity: Option<bool>,
    vesting_period_seconds: Option<u64>,
    source_nns_neuron_id: Option<u64>,
    voting_power_percentage_multiplier: u64,
    // Independent schema fields deliberately omitted from the product projection.
    followees: Vec<(u64, Followees)>,
    latest_reward_event_participation: Option<RewardEventParticipation>,
}
#[derive(CandidType, Deserialize)]
struct GovernanceError {
    error_type: i32,
    error_message: String,
}
#[derive(CandidType, Deserialize)]
enum ResultValue {
    Neuron(Neuron),
    Error(GovernanceError),
}
#[derive(CandidType, Deserialize)]
struct GetResponse {
    result: Option<ResultValue>,
}
#[derive(CandidType, Deserialize)]
struct ListResponse {
    neurons: Vec<Neuron>,
}

fn limits() -> Limits {
    Limits {
        bytes: 65536,
        header: 8192,
        types: 128,
        work: 1_000_000,
        allocation: 1_000_000,
        vector: 1024,
        skipping: 65536,
    }
}

fn neuron() -> Neuron {
    Neuron {
        id: Some(NeuronId { id: vec![255; 32] }),
        permissions: vec![
            Permission {
                principal: None,
                permission_type: vec![i32::MIN, -1, 0, i32::MAX],
            },
            Permission {
                principal: Some(Principal::anonymous()),
                permission_type: vec![],
            },
        ],
        cached_neuron_stake_e8s: u64::MAX,
        neuron_fees_e8s: 17,
        maturity_e8s_equivalent: 23,
        staked_maturity_e8s_equivalent: Some(0),
        created_timestamp_seconds: 31,
        aging_since_timestamp_seconds: u64::MAX,
        dissolve_state: Some(DissolveState::DissolveDelaySeconds(u64::MAX)),
        auto_stake_maturity: Some(false),
        vesting_period_seconds: Some(0),
        source_nns_neuron_id: Some(u64::MAX),
        voting_power_percentage_multiplier: 999,
        followees: vec![(
            u64::MAX,
            Followees {
                followees: vec![NeuronId { id: vec![0; 32] }],
            },
        )],
        latest_reward_event_participation: Some(RewardEventParticipation {
            reward_event_end_timestamp_seconds: Some(0),
            reward_shares: Some("340282366920938463463374607431768211456".parse().unwrap()),
        }),
    }
}

fn freeze(name: &str, bytes: &[u8]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine-view-wasm/fixtures")
        .join(name);
    if std::env::var_os("REGENERATE_SNS_FIXTURES").is_some() {
        std::fs::write(&path, bytes).unwrap();
    }
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

fn check_neuron(value: &sns::Neuron) {
    assert_eq!(value.id.as_ref().unwrap().id, vec![255; 32]);
    assert_eq!(value.permissions[0].principal, None);
    assert_eq!(
        value.permissions[0].permission_type,
        [i32::MIN, -1, 0, i32::MAX]
    );
    assert_eq!(
        value.permissions[1].principal,
        Some(candidate_types::Principal::anonymous())
    );
    assert_eq!(value.cached_neuron_stake_e8s, u64::MAX);
    assert_eq!(value.neuron_fees_e8s, 17);
    assert_eq!(value.maturity_e8s_equivalent, 23);
    assert_eq!(value.staked_maturity_e8s_equivalent, Some(0));
    assert_eq!(value.created_timestamp_seconds, 31);
    assert_eq!(value.aging_since_timestamp_seconds, u64::MAX);
    assert_eq!(value.auto_stake_maturity, Some(false));
    assert_eq!(value.vesting_period_seconds, Some(0));
    assert_eq!(value.source_nns_neuron_id, Some(u64::MAX));
    assert_eq!(value.voting_power_percentage_multiplier, 999);
}

#[test]
fn requests_match_independent_optional_record_schema() {
    for id in [None, Some(vec![]), Some(vec![255; 32]), Some(vec![1; 33])] {
        let value = sns::GetNeuronArgs {
            neuron_id: id.clone().map(|id| sns::NeuronId { id }),
        };
        let bytes = value.encode(4096).unwrap();
        assert_eq!(
            candid::decode_one::<GetNeuron>(&bytes).unwrap(),
            GetNeuron {
                neuron_id: id.clone().map(|id| NeuronId { id })
            }
        );
        assert_eq!(value.encode(bytes.len()).unwrap(), bytes);
        assert!(value.encode(bytes.len() - 1).is_err());
        for owner in [None, Some(candidate_types::Principal::anonymous())] {
            let value = sns::ListNeuronsArgs {
                limit: u32::MAX,
                of_principal: owner,
                start_page_at: id.clone().map(|id| sns::NeuronId { id }),
            };
            let bytes = value.encode(4096).unwrap();
            assert_eq!(
                candid::decode_one::<ListNeurons>(&bytes).unwrap(),
                ListNeurons {
                    limit: u32::MAX,
                    of_principal: owner.map(|p| Principal::from_slice(p.as_slice())),
                    start_page_at: id.clone().map(|id| NeuronId { id })
                }
            );
            assert_eq!(value.encode(bytes.len()).unwrap(), bytes);
            assert!(value.encode(bytes.len() - 1).is_err());
        }
    }
}

#[test]
fn success_errors_missing_and_raw_optionals_remain_distinct() {
    let bytes = candid::encode_one(GetResponse {
        result: Some(ResultValue::Neuron(neuron())),
    })
    .unwrap();
    freeze("sns-neuron.bin", &bytes);
    let sns::GetNeuronResult::Neuron(value) = sns::decode_get_neuron(&bytes, limits())
        .unwrap()
        .result
        .unwrap()
    else {
        panic!()
    };
    check_neuron(&value);
    assert_eq!(
        value.dissolve_state,
        Some(sns::DissolveState::DissolveDelaySeconds(u64::MAX))
    );
    let mut other = neuron();
    other.id = None;
    other.permissions = vec![];
    other.staked_maturity_e8s_equivalent = None;
    other.auto_stake_maturity = None;
    other.vesting_period_seconds = None;
    other.source_nns_neuron_id = None;
    other.dissolve_state = Some(DissolveState::WhenDissolvedTimestampSeconds(u64::MAX));
    let bytes = candid::encode_one(ListResponse {
        neurons: vec![neuron(), other.clone()],
    })
    .unwrap();
    freeze("sns-list.bin", &bytes);
    let result = sns::decode_list_neurons(&bytes, limits()).unwrap();
    check_neuron(&result.neurons[0]);
    let value = &result.neurons[1];
    assert_eq!(value.id, None);
    assert!(value.permissions.is_empty());
    assert_eq!(value.staked_maturity_e8s_equivalent, None);
    assert_eq!(value.auto_stake_maturity, None);
    assert_eq!(value.vesting_period_seconds, None);
    assert_eq!(value.source_nns_neuron_id, None);
    assert_eq!(
        value.dissolve_state,
        Some(sns::DissolveState::WhenDissolvedTimestampSeconds(u64::MAX))
    );
    other.dissolve_state = None;
    let bytes = candid::encode_one(ListResponse {
        neurons: vec![other],
    })
    .unwrap();
    assert_eq!(
        sns::decode_list_neurons(&bytes, limits()).unwrap().neurons[0].dissolve_state,
        None
    );
    for code in [i32::MIN, i32::MAX] {
        let bytes = candid::encode_one(GetResponse {
            result: Some(ResultValue::Error(GovernanceError {
                error_type: code,
                error_message: "refused\0é".into(),
            })),
        })
        .unwrap();
        if code == i32::MIN {
            freeze("sns-error.bin", &bytes);
        }
        let sns::GetNeuronResult::Error(value) = sns::decode_get_neuron(&bytes, limits())
            .unwrap()
            .result
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(value.error_type, code);
        assert_eq!(value.error_message, "refused\0é");
    }
    let bytes = candid::encode_one(GetResponse { result: None }).unwrap();
    freeze("sns-missing.bin", &bytes);
    assert_eq!(
        sns::decode_get_neuron(&bytes, limits()).unwrap().result,
        None
    );
}

fn refused_prefixes(bytes: &[u8], decode: impl Fn(&[u8], Limits) -> Result<(), Error>) {
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end], limits()).is_err(), "prefix {end}");
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert_eq!(decode(&trailing, limits()), Err(Error::Trailing));
    let mut cap = limits();
    cap.bytes = bytes.len();
    assert!(decode(bytes, cap).is_ok());
    cap.bytes -= 1;
    assert_eq!(decode(bytes, cap), Err(Error::Limit));
    // Find each minimal admitted resource ceiling independently, then check one below.
    for select in 0..6 {
        let set = |cap: &mut Limits, n| match select {
            0 => cap.header = n,
            1 => cap.types = n,
            2 => cap.work = n,
            3 => cap.allocation = n,
            4 => cap.vector = n,
            _ => cap.skipping = n,
        };
        let mut lo = 0;
        let mut hi = 1_000_000;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let mut cap = limits();
            set(&mut cap, mid);
            if decode(bytes, cap).is_ok() {
                hi = mid
            } else {
                lo = mid + 1
            }
        }
        let mut cap = limits();
        set(&mut cap, lo);
        assert!(decode(bytes, cap).is_ok());
        if lo > 0 {
            set(&mut cap, lo - 1);
            assert_eq!(decode(bytes, cap), Err(Error::Limit));
        }
    }
}

#[test]
fn budgets_wrong_types_arity_variants_and_malformed_fields_refuse() {
    let bytes = candid::encode_one(GetResponse {
        result: Some(ResultValue::Neuron(neuron())),
    })
    .unwrap();
    refused_prefixes(&bytes, |b, l| sns::decode_get_neuron(b, l).map(|_| ()));
    let bytes = candid::encode_one(ListResponse {
        neurons: vec![neuron(), neuron()],
    })
    .unwrap();
    refused_prefixes(&bytes, |b, l| sns::decode_list_neurons(b, l).map(|_| ()));
    #[derive(CandidType)]
    enum Unknown {
        Other(u64),
    }
    #[derive(CandidType)]
    struct UnknownResponse {
        result: Option<Unknown>,
    }
    let bytes = candid::encode_one(UnknownResponse {
        result: Some(Unknown::Other(0)),
    })
    .unwrap();
    assert!(sns::decode_get_neuron(&bytes, limits()).is_err());
    #[derive(CandidType)]
    struct Wrong {
        result: Option<u64>,
    }
    assert!(
        sns::decode_get_neuron(
            &candid::encode_one(Wrong { result: Some(0) }).unwrap(),
            limits()
        )
        .is_err()
    );
    assert_eq!(
        sns::decode_get_neuron(b"DIDL\0\0", limits()),
        Err(Error::Arity)
    );
    let bytes = candid::encode_args((GetResponse { result: None }, false)).unwrap();
    assert_eq!(sns::decode_get_neuron(&bytes, limits()), Err(Error::Arity));
    let bytes = candid::encode_one(GetResponse {
        result: Some(ResultValue::Error(GovernanceError {
            error_type: 1,
            error_message: "XYZ".into(),
        })),
    })
    .unwrap();
    let mut malformed = bytes;
    let i = malformed.windows(3).position(|w| w == b"XYZ").unwrap();
    malformed[i] = 255;
    assert!(sns::decode_get_neuron(&malformed, limits()).is_err());
    #[derive(CandidType)]
    struct Extra {
        result: Option<ResultValue>,
        discarded: String,
    }
    let mut bytes = candid::encode_one(Extra {
        result: None,
        discarded: "XYZ".into(),
    })
    .unwrap();
    let i = bytes.windows(3).position(|w| w == b"XYZ").unwrap();
    bytes[i] = 255;
    assert!(sns::decode_get_neuron(&bytes, limits()).is_err());
}

#[test]
fn display_metadata_is_independent_of_full_width_balances() {
    assert_eq!(ledger::encode_metadata_args(6).unwrap(), b"DIDL\0\0");
    assert!(ledger::encode_metadata_args(5).is_err());
    for n in [0u8, 8, 255] {
        let bytes = candid::encode_one(n).unwrap();
        assert_eq!(ledger::decode_decimals(&bytes, limits()).unwrap(), n);
        refused_prefixes(&bytes, |b, l| ledger::decode_decimals(b, l).map(|_| ()));
    }
    assert!(ledger::decode_decimals(&candid::encode_one(8u64).unwrap(), limits()).is_err());
    let bytes = candid::encode_one("TÖKO\0").unwrap();
    freeze("icrc-symbol.bin", &bytes);
    assert_eq!(ledger::decode_symbol(&bytes, limits()).unwrap(), "TÖKO\0");
    refused_prefixes(&bytes, |b, l| ledger::decode_symbol(b, l).map(|_| ()));
    let mut malformed = bytes;
    *malformed.last_mut().unwrap() = 255;
    assert!(ledger::decode_symbol(&malformed, limits()).is_err());
    let amount: Nat = "340282366920938463463374607431768211456".parse().unwrap();
    let bytes = candid::encode_one(&amount).unwrap();
    freeze("icrc-balance.bin", &bytes);
    assert_eq!(
        ledger::decode_amount(&bytes, limits())
            .unwrap()
            .0
            .to_string(),
        amount.0.to_string()
    );
    // Metadata failure neither narrows nor discards the separately returned balance.
    assert!(ledger::decode_symbol(&[0], limits()).is_err());
    assert_eq!(
        ledger::decode_amount(&bytes, limits())
            .unwrap()
            .0
            .to_string(),
        amount.0.to_string()
    );
    #[derive(CandidType, Deserialize)]
    struct Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    for subaccount in [None, Some([0; 32])] {
        let a = candidate_types::Account {
            owner: candidate_types::Principal::anonymous(),
            subaccount,
        };
        let bytes = ledger::encode_balance_account(&a, 1024).unwrap();
        let decoded: Account = candid::decode_one(&bytes).unwrap();
        assert_eq!(decoded.owner, Principal::anonymous());
        assert_eq!(decoded.subaccount, subaccount.map(|x| x.to_vec()));
    }
}

#[test]
fn absent_optional_fields_and_unknown_dissolve_cases_are_distinct() {
    #[derive(CandidType)]
    struct Empty {}
    let bytes = candid::encode_one(Empty {}).unwrap();
    assert_eq!(
        sns::decode_get_neuron(&bytes, limits()).unwrap().result,
        None
    );
    assert!(sns::decode_list_neurons(&bytes, limits()).is_err());

    #[derive(CandidType)]
    struct MinimalNeuron {
        permissions: Vec<MinimalPermission>,
        cached_neuron_stake_e8s: u64,
        neuron_fees_e8s: u64,
        maturity_e8s_equivalent: u64,
        created_timestamp_seconds: u64,
        aging_since_timestamp_seconds: u64,
        voting_power_percentage_multiplier: u64,
    }
    #[derive(CandidType)]
    struct MinimalPermission {
        permission_type: Vec<i32>,
    }
    #[derive(CandidType)]
    enum MinimalResult {
        Neuron(MinimalNeuron),
    }
    #[derive(CandidType)]
    struct MinimalResponse {
        result: Option<MinimalResult>,
    }
    let bytes = candid::encode_one(MinimalResponse {
        result: Some(MinimalResult::Neuron(MinimalNeuron {
            permissions: vec![MinimalPermission {
                permission_type: vec![i32::MIN],
            }],
            cached_neuron_stake_e8s: 0,
            neuron_fees_e8s: 0,
            maturity_e8s_equivalent: 0,
            created_timestamp_seconds: 0,
            aging_since_timestamp_seconds: 0,
            voting_power_percentage_multiplier: 0,
        })),
    })
    .unwrap();
    let Some(sns::GetNeuronResult::Neuron(value)) =
        sns::decode_get_neuron(&bytes, limits()).unwrap().result
    else {
        panic!()
    };
    assert!(value.id.is_none());
    assert!(value.dissolve_state.is_none());
    assert!(value.permissions[0].principal.is_none());
    // Mutate the independently encoded known variant's type-table field ID,
    // retaining its payload and valid sorted table shape.
    let mut bytes = candid::encode_one(GetResponse {
        result: Some(ResultValue::Neuron(neuron())),
    })
    .unwrap();
    fn leb(mut value: u32) -> Vec<u8> {
        let mut bytes = vec![];
        loop {
            let byte = (value & 127) as u8;
            value >>= 7;
            bytes.push(byte | if value == 0 { 0 } else { 128 });
            if value == 0 {
                return bytes;
            }
        }
    }
    let known = leb(candid::idl_hash("DissolveDelaySeconds"));
    let unknown = leb(candid::idl_hash("DissolveDelaySeconds") + 1);
    let offset = bytes.windows(known.len()).position(|b| b == known).unwrap();
    bytes[offset..offset + known.len()].copy_from_slice(&unknown);
    assert!(sns::decode_get_neuron(&bytes, limits()).is_err());
}
