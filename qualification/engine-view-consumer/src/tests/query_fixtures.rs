//! Independent producer for the versioned, replay-only Query corpus.
use crate::fixtures::{
    FullViewFrame, MarketplaceEntry, PageArgs, PageResult, Placement, RegistryCopyStatus,
};
use candid::{CandidType, Principal};
use serde::de::DeserializeOwned;
use serde_json::Value;

fn encode<T: CandidType + DeserializeOwned>(input: &Value) -> Vec<u8> {
    candid::encode_one(serde_json::from_value::<T>(input.clone()).unwrap()).unwrap()
}

#[test]
fn portable_query_corpus_matches_independent_schemas() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../query-hard-cut/portable/corpus-v1.json");
    let mut corpus: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(corpus["version"], 1);
    let regenerate = std::env::var_os("REGENERATE_QUERY_CORPUS").is_some();
    for frame in corpus["frames"].as_array_mut().unwrap() {
        let input = &frame["input"];
        let bytes = match frame["schema"].as_str().unwrap() {
            "f64" => candid::encode_one(f64::from_bits(input.as_str().unwrap().parse().unwrap()))
                .unwrap(),
            "PageResult" => encode::<PageResult>(input),
            "RegistryCopyStatus" => encode::<RegistryCopyStatus>(input),
            "Result<Placement, String>" => encode::<Result<Placement, String>>(input),
            "Vec<FullViewFrame>" => encode::<Vec<FullViewFrame>>(input),
            "Vec<MarketplaceEntry>" => encode::<Vec<MarketplaceEntry>>(input),
            "view-argument" => candid::encode_one(vec![input.as_str().unwrap()]).unwrap(),
            "page-argument" => {
                let raw: Option<Vec<u8>> = serde_json::from_value(input.clone()).unwrap();
                candid::encode_one(PageArgs {
                    limit: Some(100u64.into()),
                    start: raw.map(|bytes| Principal::from_slice(&bytes)),
                    rejoining: None,
                })
                .unwrap()
            }
            "placement-surplus" => {
                candid::encode_args((Ok::<_, String>(Placement { subnet_id: None }), 1u8)).unwrap()
            }
            "principal-text" => {
                let raw: Vec<u8> = serde_json::from_value(input.clone()).unwrap();
                Principal::from_slice(&raw).to_text().into_bytes()
            }
            other => panic!("unknown corpus schema: {other}"),
        };
        let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        if regenerate {
            frame["hex"] = hex.into();
        } else {
            assert_eq!(frame["hex"], hex, "{}", frame["schema"]);
        }
    }
    if regenerate {
        let mut bytes = serde_json::to_vec(&corpus).unwrap();
        bytes.push(b'\n');
        std::fs::write(path, bytes).unwrap();
    }
}
