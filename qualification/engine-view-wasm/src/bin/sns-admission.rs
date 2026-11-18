//! Independent SNS and ICRC frames replayed without the native Candid oracle.
#![forbid(unsafe_code)]
use ic_platform_protocol::{candid::Limits, ledger, sns};

fn main() {
    let bytes = include_bytes!("../../fixtures/sns-neuron.bin");
    let limits = Limits {
        bytes: 65536,
        header: 8192,
        types: 128,
        work: 1000000,
        allocation: 1000000,
        vector: 1024,
        skipping: 65536,
    };
    let Some(sns::GetNeuronResult::Neuron(neuron)) =
        sns::decode_get_neuron(bytes, limits).unwrap().result
    else {
        panic!()
    };
    assert_eq!(neuron.cached_neuron_stake_e8s, u64::MAX);
    assert_eq!(
        neuron.permissions[0].permission_type,
        [i32::MIN, -1, 0, i32::MAX]
    );
    assert_eq!(neuron.id.unwrap().id, [255; 32]);
    assert!(matches!(
        neuron.dissolve_state,
        Some(sns::DissolveState::DissolveDelaySeconds(u64::MAX))
    ));
    for end in 0..bytes.len() {
        assert!(sns::decode_get_neuron(&bytes[..end], limits).is_err());
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert!(sns::decode_get_neuron(&trailing, limits).is_err());
    for select in 0..7 {
        let mut bounded = limits;
        match select {
            0 => bounded.bytes = bytes.len() - 1,
            1 => bounded.header = 0,
            2 => bounded.types = 0,
            3 => bounded.work = 0,
            4 => bounded.allocation = 0,
            5 => bounded.vector = 0,
            _ => bounded.skipping = 0,
        }
        assert!(sns::decode_get_neuron(bytes, bounded).is_err());
    }
    assert!(
        !sns::decode_list_neurons(include_bytes!("../../fixtures/sns-list.bin"), limits)
            .unwrap()
            .neurons
            .is_empty()
    );
    assert!(matches!(
        sns::decode_get_neuron(include_bytes!("../../fixtures/sns-error.bin"), limits)
            .unwrap()
            .result,
        Some(sns::GetNeuronResult::Error(_))
    ));
    assert!(
        sns::decode_get_neuron(include_bytes!("../../fixtures/sns-missing.bin"), limits)
            .unwrap()
            .result
            .is_none()
    );
    assert_eq!(
        ledger::decode_symbol(include_bytes!("../../fixtures/icrc-symbol.bin"), limits).unwrap(),
        "TÖKO\0"
    );
    assert_eq!(
        ledger::decode_amount(include_bytes!("../../fixtures/icrc-balance.bin"), limits)
            .unwrap()
            .0
            .to_string(),
        "340282366920938463463374607431768211456"
    );
    assert_eq!(ledger::encode_metadata_args(6).unwrap(), b"DIDL\0\0");
    assert!(ledger::encode_metadata_args(5).is_err());
    assert_eq!(
        ledger::decode_decimals(b"DIDL\0\x01\x7b\xff", limits).unwrap(),
        255
    );
    let request = sns::GetNeuronArgs { neuron_id: None };
    let encoded = request.encode(1024).unwrap();
    assert!(request.encode(encoded.len() - 1).is_err());
}
