//! Focused raw_rand interoperability and unchanged Runtime behavior.
mod pocketic_support;
use ic_platform::{candid_codec, raw_rand};
use ic_testkit::pocket_ic::CanisterId as Principal;
use std::time::Duration;

fn main() {
    pocketic_support::runtime().block_on(run());
}

async fn run() {
    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let pic = builder.with_application_subnet().build().await.unwrap();
    for path in std::env::args().skip(1) {
        let id = pic.create_canister().await.unwrap();
        pic.add_cycles(id, 10_000_000_000_000).await.unwrap();
        pic.install_canister(id, std::fs::read(&path).unwrap(), vec![], None)
            .await
            .unwrap();
        for _ in 0..3 {
            let bytes = pic
                .update_call(id, Principal::anonymous(), "entropy", vec![0])
                .await
                .unwrap();
            let random = raw_rand::decode(&bytes).unwrap();
            let reference: Vec<u8> = candid_codec::decoder::decode_one(
                &bytes,
                candid_codec::decoder::Limits {
                    bytes: 256,
                    header: 64,
                    types: 8,
                    work: 4096,
                    allocation: 256,
                },
            )
            .unwrap();
            assert_eq!(reference, random);
            assert_eq!(bytes, candid_codec::encode_one(&reference, 256).unwrap());
        }
        assert_eq!(
            pic.update_call(id, Principal::anonymous(), "entropy", vec![1])
                .await
                .unwrap(),
            [1]
        );
        assert!(
            pic.update_call(id, Principal::anonymous(), "entropy", vec![2])
                .await
                .is_err()
        );
        assert_eq!(
            pic.query_call(id, Principal::anonymous(), "cleaned", vec![])
                .await
                .unwrap(),
            [1]
        );
        assert_eq!(
            pic.update_call(id, Principal::anonymous(), "entropy", vec![3])
                .await
                .unwrap(),
            [2]
        );
        let bytes = pic
            .update_call(id, Principal::anonymous(), "entropy", vec![0])
            .await
            .unwrap();
        raw_rand::decode(&bytes).unwrap();
        println!(
            "{path}: raw_rand producer bytes, structured rejection, callback-trap cleanup, cancellation and subsequent call passed"
        );
    }
    pic.drop().await.unwrap();
    drop(server);
}
