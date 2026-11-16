use super::{Budget, Error, Field, FieldValue, Limits, Reader, RegistryError};

/// Unauthenticated Registry response preserving inline content and mutation evidence.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct GetValueResponse<'a> {
    /// Present source error, including a present empty error message.
    pub error: Option<RegistryError<'a>>,
    /// Version at which the key was last mutated, not the requested snapshot version.
    pub version: u64,
    /// Inline value; missing differs from present empty.
    pub value: Option<&'a [u8]>,
    /// Raw mutation timestamp, not an authenticated clock.
    pub timestamp_nanoseconds: u64,
}
impl<'a> Field<'a> for GetValueResponse<'a> {
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error> {
        let FieldValue::Bytes(bytes) = value else {
            return Err(Error::Malformed);
        };
        let mut reader = Reader::new(bytes);
        let mut seen = 0u8;
        while let Some((number, value)) = reader.next(&mut budget.fields)? {
            if (1..=5).contains(&number) {
                if seen & (1 << number) != 0 {
                    return Err(Error::DuplicateField);
                }
                seen |= 1 << number;
            }
            match number {
                1 => self.error.read(value, budget)?,
                2 => self.version.read(value, budget)?,
                3 => self.value.read(value, budget)?,
                4 => return Err(Error::ChunkedValue),
                5 => self.timestamp_nanoseconds.read(value, budget)?,
                _ => (),
            }
        }
        Ok(())
    }
}
/// Decode an inline response with one aggregate field budget including its error.
///
/// # Errors
/// Refuses malformed/duplicate input, unsupported chunked content and exhausted limits.
pub fn decode_get_value_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<GetValueResponse<'_>, Error> {
    super::decode(bytes, limits)
}

/// Decode the raw latest Registry version; absent proto3 scalar defaults to zero.
///
/// # Errors
/// Refuses malformed/duplicate input and exhausted byte/field limits.
pub fn decode_get_latest_version_response(bytes: &[u8], limits: Limits) -> Result<u64, Error> {
    if bytes.len() > limits.bytes {
        return Err(Error::ByteLimit);
    }
    let mut reader = Reader::new(bytes);
    let mut remaining = limits.fields;
    let mut seen = false;
    let mut result = 0;
    while let Some((number, value)) = reader.next(&mut remaining)? {
        if number == 1 {
            if seen {
                return Err(Error::DuplicateField);
            }
            seen = true;
            let FieldValue::Integer(n) = value else {
                return Err(Error::Malformed);
            };
            result = n;
        }
    }
    Ok(result)
}
/// Decode subnet IDs as borrowed byte strings in source order, without identity admission.
///
/// # Errors
/// Refuses malformed input or byte/field/item/allocation exhaustion.
pub fn decode_subnet_list(bytes: &[u8], limits: Limits) -> Result<Vec<&[u8]>, Error> {
    if bytes.len() > limits.bytes {
        return Err(Error::ByteLimit);
    }
    let mut reader = Reader::new(bytes);
    let mut budget = Budget {
        fields: limits.fields,
        allocation: limits.allocation,
        items: limits.items,
    };
    let mut result = Vec::new();
    while let Some((number, value)) = reader.next(&mut budget.fields)? {
        if number == 2 {
            result.read(value, &mut budget)?;
        }
    }
    Ok(result)
}
fn width(n: u64) -> usize {
    if n == 0 {
        1
    } else {
        usize::try_from(n.ilog2() / 7 + 1).expect("u64 width fits usize")
    }
}
fn integer(mut n: u64, output: &mut Vec<u8>) {
    while n >= 128 {
        output.push(n as u8 | 128);
        n >>= 7;
    }
    output.push(n as u8);
}
/// Encode a key and optional UInt64Value snapshot version with a preallocation bound.
///
/// # Errors
/// Returns OutputLimit before allocation if the complete request cannot fit.
pub fn encode_get_value_request(
    key: &[u8],
    version: Option<u64>,
    maximum: usize,
) -> Result<Vec<u8>, Error> {
    let key_len = u64::try_from(key.len()).map_err(|_| Error::OutputLimit)?;
    let wrapped = version.map_or(0, |n| if n == 0 { 2 } else { 3 + width(n) });
    let size = key
        .len()
        .checked_add(1 + width(key_len))
        .and_then(|n| n.checked_add(wrapped))
        .ok_or(Error::OutputLimit)?;
    if size > maximum {
        return Err(Error::OutputLimit);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| Error::AllocationLimit)?;
    if let Some(n) = version {
        output.push(10);
        if n == 0 {
            output.push(0);
        } else {
            output.push((1 + width(n)) as u8);
            output.push(8);
            integer(n, &mut output);
        }
    }
    output.push(18);
    integer(key_len, &mut output);
    output.extend_from_slice(key);
    Ok(output)
}
