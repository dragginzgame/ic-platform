//! Native export generated beside the mounted declarations, with an independent DID.
#![cfg(not(target_family = "wasm"))]
use candid_parser::utils::{CandidSource, service_equal};
use ic_platform::{
    Principal,
    candid_codec::{
        Nat,
        service::{self, Method, Mode},
    },
};

#[derive(Debug)]
struct Certificate {
    method: String,
    blob_hash: String,
}
ic_platform::candid_record!(Certificate {156956385 => method: String, 278101296 => blob_hash: String});

// One catalogue emits executable host fixtures and descriptors together. Real
// consumers use their existing mount catalogue, including its external wire names.
macro_rules! mount {
    ($( $mode:ident fn $name:ident($($arg:ident : $ty:ty),*) -> ($($result:ty),*) $body:block )*) => {
        $(#[expect(non_snake_case, reason = "frozen external Caffeine method names")] fn $name($($arg:$ty),*) -> ($($result,)*) $body)*
        fn methods() -> Vec<Method> { vec![$(Method::new::<($($ty,)*), ($($result,)*)>(stringify!($name), Mode::$mode)),*] }
    };
}
mount! {
    Query fn _immutableObjectStorageBlobsAreLive(roots: Vec<Vec<u8>>) -> (Vec<bool>) { (vec![true; roots.len()],) }
    Query fn _immutableObjectStorageBlobsToDelete() -> (Vec<Vec<u8>>) { (Vec::new(),) }
    Update fn _immutableObjectStorageConfirmBlobDeletion(roots: Vec<Vec<u8>>) -> () { drop(roots); }
    Update fn _immutableObjectStorageCreateCertificate(hash: String) -> (Certificate) { (Certificate {method: "external".into(), blob_hash: hash},) }
}
const FROZEN: &str = r#"
type Certificate = record { method : text; blob_hash : text };
service : {
  _immutableObjectStorageBlobsAreLive : (vec blob) -> (vec bool) query;
  _immutableObjectStorageBlobsToDelete : () -> (vec blob) query;
  _immutableObjectStorageConfirmBlobDeletion : (vec blob) -> ();
  _immutableObjectStorageCreateCertificate : (text) -> (Certificate);
}
"#;
fn equal(actual: &str, expected: &str) -> bool {
    service_equal(CandidSource::Text(actual), CandidSource::Text(expected)).is_ok()
}
#[test]
fn mounted_provider_conforms_and_mismatches_refuse() {
    assert_eq!(
        _immutableObjectStorageBlobsAreLive(vec![vec![1]]),
        (vec![true],)
    );
    assert!(_immutableObjectStorageBlobsToDelete().0.is_empty());
    _immutableObjectStorageConfirmBlobDeletion(vec![]);
    assert_eq!(
        _immutableObjectStorageCreateCertificate("hash".into())
            .0
            .blob_hash,
        "hash"
    );
    let actual = service::export(&methods(), 16384).unwrap();
    assert!(equal(&actual, FROZEN));
    let mutations = [
        FROZEN.replace("vec bool", "vec nat8"),
        FROZEN.replacen("query", "", 1),
        FROZEN.replace("(text)", "(text, principal)"),
        FROZEN.replace("-> (Certificate)", "-> ()"),
        FROZEN.replace("_immutableObjectStorageBlobsAreLive", "wrong_name"),
        "service : { broken : ( }".into(),
    ];
    for changed in &mutations {
        assert!(!equal(&actual, changed));
    }
    // An explicitly selected scratch directory lets the didc CLI qualify the
    // same actual/mismatch files. Ordinary cargo tests need no executable setup.
    if let Ok(directory) = std::env::var("PLATFORM_SERVICE_EVIDENCE") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("actual.did"), actual).unwrap();
        std::fs::write(directory.join("frozen.did"), FROZEN).unwrap();
        for (index, changed) in mutations.iter().enumerate() {
            std::fs::write(directory.join(format!("mismatch-{index}.did")), changed).unwrap();
        }
    }
}
#[derive(Debug)]
struct Identity {
    owner: Principal,
    amount: Nat,
}
ic_platform::candid_record!(Identity {947296307 => owner: Principal, 3573748184 => amount: Nat});
#[test]
fn canonical_identity_and_nat_keep_their_external_schema() {
    let actual = service::export(
        &[Method::new::<(Identity,), (Principal, Nat)>(
            "identity",
            Mode::Query,
        )],
        4096,
    )
    .unwrap();
    assert!(equal(
        &actual,
        "service : { identity : (record { owner : principal; amount : nat }) -> (principal, nat) query }"
    ));
}
