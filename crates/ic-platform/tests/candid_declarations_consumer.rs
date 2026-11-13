//! Standalone public declarations: this file also compiles as a native/Wasm library.
use ic_platform::candid_codec::{
    Nat,
    decoder::{Limits, decode_one},
    encode_one,
};

ic_platform::candid_record! {
    /// Index item with explicit protocol labels.
    #[derive(Debug, Eq, PartialEq)]
    pub struct IcrcIndexTransactionWithId {
        /// Transaction index.
        0 => pub id: Nat,
        /// Minimal test payload.
        1 => pub transaction: bool,
    }
}
ic_platform::candid_record! {
    /// Query's index envelope can be declared once, without Serde decoding.
    #[derive(Debug, Eq, PartialEq)]
    pub struct IcrcIndexTransactions {
        /// Unbounded balance, preserving the IC's Nat range.
        596_483_356 => pub balance: Nat,
        /// Selected transactions.
        3_331_539_157 => pub transactions: Vec<IcrcIndexTransactionWithId>,
        3_582_638_454 => pub(crate) oldest_tx_id: Option<Nat>,
    }
}
ic_platform::candid_variant! {
    /// All supported payload forms, including recursion and a tuple payload.
    #[derive(Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub enum Response {
        /// Unit payload.
        0 => Empty,
        /// Record payload.
        1 => Value(IcrcIndexTransactions),
        /// Tuple payload.
        2 => Pair((u64, String)),
        /// Named recursive payload.
        3 => Nested {
            /// Recursive children.
            0 => values: Vec<Response>,
            /// Optional tuple.
            1 => next: Option<(u64, String)>,
        },
    }
}

// Shape-changing configuration gates the invocation as one Rust/wire unit.
#[cfg(any())]
ic_platform::candid_record! { pub struct NotSelected { 0 => unavailable: MissingType } }

/// Exercise generated mappings without upstream codec or Serde traits.
/// # Panics
/// Panics if a consumer contract assertion fails.
pub fn exercise() {
    let value = Response::Nested {
        values: vec![Response::Value(IcrcIndexTransactions {
            balance: "340282366920938463463374607431768211456".parse().unwrap(),
            transactions: vec![IcrcIndexTransactionWithId {
                id: Nat::from(u128::MAX),
                transaction: true,
            }],
            oldest_tx_id: None,
        })],
        next: Some((42, "message".into())),
    };
    let bytes = encode_one(&value, 4096).unwrap();
    let limits = Limits {
        bytes: 4096,
        header: 1024,
        types: 64,
        work: 100_000,
        allocation: 16_384,
    };
    assert_eq!(decode_one::<Response>(&bytes, limits), Ok(value));
}

#[test]
fn declaration_only_consumer() {
    exercise();
}
