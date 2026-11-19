//! Independent installed system and generated endpoint probe for Auth's requirements.

use candidate_runtime::{entrypoints, executor::EntryError, system};

/// Admit every caller to this qualification fixture.
fn admit() -> Result<(), u8> {
    Ok(())
}

fn unit(bytes: &[u8]) -> Result<(), u8> {
    if bytes.is_empty() { Ok(()) } else { Err(1) }
}

fn bytes(input: &[u8]) -> Result<Vec<u8>, u8> {
    Ok(input.to_vec())
}

fn fail<T: std::fmt::Debug>(error: T) -> ! {
    panic!("system fixture refused: {error:?}")
}

fn encode(value: Vec<u8>) -> Result<Vec<u8>, u8> {
    Ok(value)
}

fn no_task() {
    assert_eq!(system::reply(&[]), Err(system::ReplyError::NoActiveTask));
    let nested = entrypoints::lifecycle(0, false, admit, unit, |()| panic!("nested lifecycle"));
    assert!(matches!(
        nested,
        Err(entrypoints::UpdateError::Entry(EntryError::NestedEntry))
    ));
}

/// Set the initial certified root.
#[candidate_runtime::init(
    runtime = candidate_runtime,
    guard = admit,
    decode = unit,
    max_bytes = 0,
    on_error = fail
)]
fn initialize((): ()) {
    no_task();
    system::set_certified_data(&[7; 32]);
}

/// Set the certified root for the pre-upgrade observation.
#[candidate_runtime::pre_upgrade(
    runtime = candidate_runtime,
    guard = admit,
    decode = unit,
    max_bytes = 0,
    on_error = fail
)]
fn before_upgrade((): ()) {
    no_task();
    system::set_certified_data(&[8; 32]);
}

/// Migrate the fixture timer record and set the upgraded certified root.
#[candidate_runtime::post_upgrade(
    runtime = candidate_runtime,
    guard = admit,
    decode = unit,
    max_bytes = 0,
    on_error = fail
)]
fn after_upgrade((): ()) {
    super::timer::migrate_record();
    no_task();
    system::set_certified_data(&[9; 32]);
}

/// Read caller, canister, time, controller and certificate observations.
#[candidate_runtime::query(
    runtime = candidate_runtime,
    name = "system_observe",
    guard = admit,
    decode = unit,
    max_bytes = 0,
    encode = encode,
    on_error = fail
)]
fn observe((): ()) -> Vec<u8> {
    no_task();
    let caller = system::caller();
    let own = system::canister_self();
    let mut result = vec![own.len() as u8];
    result.extend_from_slice(own.as_slice());
    result.push(caller.len() as u8);
    result.extend_from_slice(caller.as_slice());
    result.extend_from_slice(&system::time().to_le_bytes());
    result.push(u8::from(system::is_controller(&caller)));
    match system::data_certificate(1_048_576).unwrap() {
        None => result.push(0),
        Some(certificate) => {
            result.push(1);
            assert_eq!(
                system::data_certificate(certificate.len())
                    .unwrap()
                    .as_ref(),
                Some(&certificate)
            );
            if !certificate.is_empty() {
                assert_eq!(
                    system::data_certificate(certificate.len() - 1),
                    Err(system::CertificateTooLarge {
                        actual: certificate.len(),
                        maximum: certificate.len() - 1
                    })
                );
            }
            result.extend(certificate);
        }
    }
    result
}

/// Publish a 32-byte certified root and return it.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "system_set",
    guard = admit,
    decode = bytes,
    max_bytes = 32,
    encode = encode,
    on_error = fail
)]
fn publish(root: Vec<u8>) -> Vec<u8> {
    let root: [u8; 32] = root.try_into().unwrap();
    assert_eq!(system::data_certificate(0), Ok(None));
    system::set_certified_data(&root);
    root.to_vec()
}

fn refuse_encoding(_: Vec<u8>) -> Result<Vec<u8>, u8> {
    Err(6)
}

/// Exercise rollback when the reply encoder refuses its result.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "system_encode_refusal",
    guard = admit,
    decode = unit,
    max_bytes = 0,
    encode = refuse_encoding,
    on_error = fail
)]
fn rollback_encoding((): ()) -> Vec<u8> {
    system::set_certified_data(&[99; 32]);
    Vec::new()
}

/// Return the async handler result through the automatic reply encoder.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "system_async",
    guard = admit,
    decode = unit,
    max_bytes = 0,
    encode = encode,
    on_error = fail
)]
async fn automatic_async((): ()) -> Vec<u8> {
    vec![42]
}
