//! Deterministic malformed-input coverage shared with the CBOR fuzzer.
mod fixtures {
    pub mod cbor_malformed;
}

#[test]
fn malformed_cbor_smoke_corpus() {
    let seeds: &[&[u8]] = &[
        b"",
        b"\xff",
        b"\xf6",
        b"\x81\x00",
        b"\x82\x03\x40",
        b"\x82\x04\x58\x20",
        b"\xa2\x64tree\x81\x00\x69signature\x40",
        b"\xa2\x64tree\x81\x00\x64tree\x81\x00",
        b"\x5b\xff\xff\xff\xff\xff\xff\xff\xff",
        b"\x9f\x03\x5f\x41a\xff\xff",
    ];
    for seed in seeds {
        fixtures::cbor_malformed::exercise(seed);
        let mut trailing = seed.to_vec();
        trailing.push(0);
        fixtures::cbor_malformed::exercise(&trailing);
        for index in 0..seed.len() {
            fixtures::cbor_malformed::exercise(&seed[..index]);
            let mut mutated = seed.to_vec();
            mutated[index] ^= 0xff;
            fixtures::cbor_malformed::exercise(&mutated);
        }
    }
    for depth in [16, 127, 128, 256] {
        let mut nested = vec![0x81; depth];
        nested.push(0);
        fixtures::cbor_malformed::exercise(&nested);
    }
}
