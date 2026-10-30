use candid::Principal;
use ic_management_canister_types::CanisterStatusResult;
use pocket_ic::PocketIcBuilder;
fn main() {
    let root = std::env::args().nth(1).expect("experiment root");
    let pic = PocketIcBuilder::new().with_application_subnet().build();
    let ids = ["baseline", "reader", "compact"].map(|name| {
        let id = pic.create_canister();
        pic.add_cycles(id, 1_000_000_000_000_000);
        pic.install_canister(
            id,
            std::fs::read(format!("{root}/{name}.wasm")).unwrap(),
            vec![],
            None,
        );
        id
    });
    let s = std::fs::read_to_string(format!(
        "{root}/baseline-harness/src/tests/fixtures/management-snapshot-status.hex"
    ))
    .unwrap();
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let wire: Vec<u8> = s
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    let values: Vec<CanisterStatusResult> = candid::decode_one(&wire).unwrap();
    let mut valid = 0;
    let mut cases = Vec::new();
    for v in values {
        let b = candid::encode_one(v).unwrap();
        for id in ids {
            assert_eq!(
                pic.query_call(id, Principal::anonymous(), "status", b.clone())
                    .unwrap(),
                b
            );
        }
        valid += 1;
        for n in [0, 1, 4, 5, 8, b.len() / 2, b.len() - 1] {
            cases.push(b[..n].to_vec());
        }
        for pos in 0..b.len().min(80) {
            let mut m = b.clone();
            m[pos] = 0xff;
            cases.push(m);
        }
    }
    let mut rejected = 0;
    let mut accepted = 0;
    for b in &cases {
        let replies = ids.map(|id| {
            pic.query_call(id, Principal::anonymous(), "status", b.clone())
                .unwrap()
        });
        let ok = replies[0].starts_with(b"DIDL");
        for reply in &replies[1..] {
            assert_eq!(reply.starts_with(b"DIDL"), ok);
            if ok {
                assert_eq!(reply, &replies[0]);
            }
        }
        if ok {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    println!(
        "three installed variants: {valid} frozen status replies; {rejected} matched refusals; {accepted} matching accepted mutations"
    );
}
