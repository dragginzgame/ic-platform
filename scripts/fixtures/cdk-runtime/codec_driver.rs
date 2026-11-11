//! Installed codec round trips using independent Candid-generated request bytes.
mod pocketic_support;
use ic_testkit::pocket_ic::CanisterId;
use std::{collections::BTreeMap, time::Duration};

fn input(profile: &str) -> Vec<u8> {
    match profile {
        "blob" => candid::encode_one(vec![0x5au8; 4096]),
        "words" => candid::encode_one(vec![u64::MAX; 1024]),
        "map" => candid::encode_one(
            (0..128)
                .map(|i| (format!("key-{i:04}"), i as u64))
                .collect::<BTreeMap<_, _>>(),
        ),
        "records" => candid::encode_one(
            (0..128)
                .map(|i| (i as u64, Some("record".to_owned())))
                .collect::<Vec<_>>(),
        ),
        "options" => candid::encode_one(Some(Ok::<_, String>(vec![42u64; 128]))),
        "bigints" => candid::encode_one((
            candid::Nat(candid::Nat::from(1u8).0 << 8192),
            candid::Int(-(candid::Int::from(1i8).0 << 8192usize)),
        )),
        _ => panic!("unsupported codec workload {profile}"),
    }
    .unwrap()
}

fn main() {
    pocketic_support::runtime().block_on(run());
}

async fn run() {
    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let pic = builder.with_application_subnet().build_async().await;
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(args.len().is_multiple_of(3));
    println!("arm\tprofile\tsample\tinstructions");
    for [arm, profile, path] in args.as_chunks::<3>().0 {
        let request = input(profile);
        let id = pic.create_canister().await;
        pic.add_cycles(id, 10_000_000_000_000).await;
        pic.install_canister(id, std::fs::read(path).unwrap(), vec![], None)
            .await;
        for sample in 0..4 {
            let response = pic
                .update_call(id, CanisterId::anonymous(), "measure", request.clone())
                .await
                .unwrap();
            assert_eq!(
                &response[8..],
                request,
                "{arm} {profile}: canonical round-trip bytes"
            );
            let instructions = u64::from_le_bytes(response[..8].try_into().unwrap());
            println!("{arm}\t{profile}\t{sample}\t{instructions}");
        }
    }
    pic.drop().await;
    drop(server);
}
