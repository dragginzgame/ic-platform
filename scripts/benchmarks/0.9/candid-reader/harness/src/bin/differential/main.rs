#[cfg(feature = "baseline-reader")]
use binrw::BinRead;
use candid::{
    CandidType, Deserialize, Reserved,
    de::{DecoderConfig, IDLDeserialize},
};
use std::io::{self, BufRead, Cursor};
fn hex(b: &[u8]) -> String {
    b.iter().map(|v| format!("{v:02x}")).collect()
}
fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn typed<T: for<'de> Deserialize<'de> + CandidType>(
    b: &[u8],
    c: &DecoderConfig,
) -> Result<String, candid::Error> {
    let mut d = IDLDeserialize::new_with_config(b, c)?;
    let v = d.get_value::<T>()?;
    d.done()?;
    Ok(format!(
        "{} {:?} {:?}",
        hex(&candid::encode_one(v)?),
        d.get_config().compute_cost(c).decoding_quota,
        d.get_config().compute_cost(c).skipping_quota
    ))
}
fn main() {
    if std::env::args().nth(1).as_deref() == Some("generate") {
        generate();
        return;
    }
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut fields = line.split('|');
        let kind = fields.next().unwrap();
        let bound: usize = fields.next().unwrap().parse().unwrap();
        let b = bytes(fields.next().unwrap());
        let mut c = DecoderConfig::new();
        c.set_max_header_len(bound)
            .set_max_type_len(100)
            .set_decoding_quota(100_000)
            .set_skipping_quota(100_000)
            .set_full_error_message(false);
        let result = match kind {
            "header" => {
                let mut cursor = Cursor::new(b.as_slice());
                candid::binary_parser::Header::read_le_args(&mut cursor, (Some(100),))
                    .map_err(candid::Error::from)
                    .and_then(|h| h.to_types().map_err(candid::Error::from))
                    .map(|(e, a)| format!("{} {e:?} {a:?}", cursor.position()))
            }
            "numbers" => typed::<Vec<(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64)>>(&b, &c),
            "floats" => typed::<Vec<f64>>(&b, &c),
            "nat" => typed::<candid::Nat>(&b, &c),
            "principal" => typed::<candid::Principal>(&b, &c),
            "status" => {
                typed::<ic_platform::management_canister_types::CanisterStatusResult>(&b, &c)
            }
            "record" => typed::<Vec<(u32, bool, String)>>(&b, &c),
            _ => typed::<Reserved>(&b, &c),
        };
        match result {
            Ok(v) => println!("OK {v}"),
            Err(e) => {
                let text = e.to_string();
                println!(
                    "ERR {}",
                    if text.contains("header exceeds the limit") {
                        "header-limit"
                    } else {
                        "decode"
                    }
                );
            }
        }
    }
}

fn generate() {
    fn emit<T: CandidType>(kind: &str, v: T) {
        println!("{kind}|65536|{}", hex(&candid::encode_one(v).unwrap()));
    }
    emit(
        "numbers",
        vec![
            (0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0.0),
            (
                u8::MAX,
                u16::MAX,
                u32::MAX,
                u64::MAX,
                i8::MIN,
                i16::MIN,
                i32::MIN,
                i64::MIN,
                f32::INFINITY,
                f64::NEG_INFINITY,
            ),
            (
                1,
                2,
                3,
                4,
                i8::MAX,
                i16::MAX,
                i32::MAX,
                i64::MAX,
                -0.0,
                f64::NAN,
            ),
        ],
    );
    emit(
        "floats",
        vec![
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MIN_POSITIVE,
            1.25,
        ],
    );
    emit(
        "nat",
        "340282366920938463463374607431768211456"
            .parse::<candid::Nat>()
            .unwrap(),
    );
    emit(
        "record",
        vec![
            (42u32, true, "abc".to_string()),
            (u32::MAX, false, "🙂".to_string()),
        ],
    );
    emit("principal", candid::Principal::anonymous());
    emit("principal", candid::Principal::from_slice(&[255; 29]));
    let s = include_str!("../../tests/fixtures/management-snapshot-status.hex");
    let wire = bytes(&s.split_whitespace().collect::<String>());
    let values: Vec<ic_platform::management_canister_types::CanisterStatusResult> =
        candid::decode_one(&wire).unwrap();
    for v in values {
        emit("status", v);
    }
}
