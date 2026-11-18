pub fn guard() -> Result<(), u8> {
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>, u8> {
    Ok(bytes.to_vec())
}

pub fn fail(error: candidate_runtime::entrypoints::UpdateError<u8, u8>) -> ! {
    panic!("{error:?}")
}
