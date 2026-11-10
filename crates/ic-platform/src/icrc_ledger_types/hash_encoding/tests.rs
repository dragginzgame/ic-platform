use crate::candid_codec::Nat;
use num_bigint::BigUint;
use std::io::{self, Write};

fn check_against_oracle(value: BigUint) {
    let oracle: candid::Nat = value
        .to_string()
        .parse()
        .expect("the independently supplied oracle accepts decimal naturals");
    let mut expected = Vec::new();
    oracle.encode(&mut expected).expect("Vec writes succeed");

    let mut actual = Vec::new();
    Nat(value.clone())
        .encode(&mut actual)
        .expect("Vec writes succeed");
    assert_eq!(actual, expected);
    assert_eq!(
        actual.len(),
        usize::try_from(value.bits().max(1).div_ceil(7)).expect("fixture length fits usize")
    );
    let (last, preceding) = actual.split_last().expect("even zero emits one byte");
    assert_eq!(last & 0x80, 0);
    assert!(preceding.iter().all(|byte| byte & 0x80 != 0));
    if actual.len() > 1 {
        assert_ne!(*last, 0, "the final group cannot be redundant");
    }
}

#[test]
fn small_payloads_have_the_specified_seven_bit_groups() {
    for (number, expected) in [
        (0_u64, vec![0]),
        (1, vec![1]),
        (126, vec![126]),
        (127, vec![127]),
        (128, vec![0x80, 1]),
        (129, vec![0x81, 1]),
        (16_383, vec![0xff, 0x7f]),
        (16_384, vec![0x80, 0x80, 1]),
    ] {
        let mut actual = Vec::new();
        Nat(BigUint::from(number))
            .encode(&mut actual)
            .expect("Vec writes succeed");
        assert_eq!(actual, expected);
    }
}

#[test]
fn power_boundaries_and_large_magnitudes_match_the_oracle() {
    check_against_oracle(BigUint::from(0_u8));
    for bits in [7_usize, 14, 31, 32, 33, 63, 64, 65, 8192] {
        let power = BigUint::from(1_u8) << bits;
        check_against_oracle(&power - BigUint::from(1_u8));
        check_against_oracle(power.clone());
        check_against_oracle(power + BigUint::from(1_u8));
    }

    let mut scattered = BigUint::from(1_u8) << 8192_usize;
    for bit in (0..8192_usize).step_by(31) {
        scattered += BigUint::from(1_u8) << bit;
    }
    check_against_oracle(scattered);
}

#[test]
fn encoding_appends_through_an_unsized_writer() {
    let mut bytes = vec![0xa5, 0x5a];
    let writer: &mut dyn Write = &mut bytes;
    Nat(BigUint::from(128_u16))
        .encode(writer)
        .expect("Vec writes succeed");
    assert_eq!(bytes, [0xa5, 0x5a, 0x80, 1]);
}

struct ShortWriter {
    bytes: Vec<u8>,
    interruptions: usize,
}

impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.interruptions != 0 {
            self.interruptions -= 1;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        let accepted = bytes.len().min(3);
        self.bytes.extend_from_slice(&bytes[..accepted]);
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("encoding must not flush")
    }
}

#[test]
fn short_and_interrupted_writes_complete_the_payload() {
    let value = BigUint::from(1_u8) << 8192_usize;
    let mut writer = ShortWriter {
        bytes: vec![0xa5],
        interruptions: 4,
    };
    Nat(value).encode(&mut writer).expect("writes recover");

    // 8192 = 1170 * 7 + 2: 1170 zero groups precede the final value 4.
    let mut expected = vec![0xa5];
    expected.extend(std::iter::repeat_n(0x80, 1170));
    expected.push(4);
    assert_eq!(writer.bytes, expected);
    assert_eq!(writer.interruptions, 0);
}

struct StalledWriter;

impl Write for StalledWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("encoding must not flush")
    }
}

#[test]
fn zero_progress_returns_write_zero() {
    let error = Nat(BigUint::from(0_u8))
        .encode(&mut StalledWriter)
        .expect_err("zero progress cannot complete the payload");
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

struct FailingWriter {
    bytes: Vec<u8>,
    remaining: usize,
}

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        let accepted = bytes.len().min(self.remaining);
        self.bytes.extend_from_slice(&bytes[..accepted]);
        self.remaining -= accepted;
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("encoding must not flush")
    }
}

#[test]
fn failure_preserves_its_kind_and_a_correct_partial_prefix() {
    let value = (BigUint::from(1_u8) << 2048_usize) - BigUint::from(1_u8);
    let mut writer = FailingWriter {
        bytes: Vec::new(),
        remaining: 137,
    };
    let error = Nat(value)
        .encode(&mut writer)
        .expect_err("the writer fails before the payload is complete");
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(writer.bytes, vec![0xff; 137]);
}
