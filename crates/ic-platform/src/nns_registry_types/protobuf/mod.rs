//! Bounded protobuf adapters for Registry `get_value` and subnet-list records.
//!
//! Independently authored from the pinned schemas in the external contract guide.
//! This selected codec accepts inline values only: chunk references fail explicitly.
//! It borrows response blobs, rejects duplicate singular fields and unsupported wire
//! groups, and skips unknown scalar/blob fields within the caller's byte/field bounds.
//! Decoding authenticates nothing. Callers own endpoint, key, version and retry policy.

use thiserror::Error;

/// Explicit admission limits; there are no implicit defaults.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum encoded input bytes.
    pub bytes: usize,
    /// Maximum fields across the envelope and supported nested messages.
    pub fields: usize,
    /// Maximum allocation for the subnet entry-reference vector.
    pub allocation: usize,
    /// Maximum subnet entries.
    pub items: usize,
}

/// Refusals from the selected protobuf contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// Input or output exceeds the explicit byte budget.
    #[error("Registry protobuf byte limit exceeded")]
    ByteLimit,
    /// Too many fields, including unknown fields and nested error fields.
    #[error("Registry protobuf field limit exceeded")]
    FieldLimit,
    /// The subnet vector would exceed its allocation budget.
    #[error("Registry protobuf allocation limit exceeded")]
    AllocationLimit,
    /// The subnet vector would exceed its entry budget.
    #[error("Registry protobuf entry limit exceeded")]
    ItemLimit,
    /// Truncated, overflowing, mistyped or otherwise invalid protobuf.
    #[error("malformed Registry protobuf")]
    Malformed,
    /// This selected codec does not support deprecated protobuf groups.
    #[error("unsupported Registry protobuf wire type")]
    UnsupportedWireType,
    /// Multiple occurrences of a singular field are ambiguous to this adapter.
    #[error("duplicate Registry protobuf singular field")]
    DuplicateField,
    /// Fetching and authenticating chunk references requires a separate contract.
    #[error("chunked Registry values are not supported")]
    ChunkedValue,
}

/// Borrowed external Registry application error; numeric codes are preserved.
#[derive(Debug, PartialEq, Eq)]
pub struct RegistryError<'a> {
    /// External protobuf enum discriminant (including unknown values).
    pub code: i32,
    /// UTF-8 source reason, without normalization or interpretation.
    pub reason: &'a str,
    /// Source key bytes.
    pub key: &'a [u8],
}

/// Supported inline projection of `HighCapacityRegistryGetValueResponse`.
#[derive(Debug, PartialEq, Eq)]
pub struct GetValueResponse<'a> {
    /// Source application error, if present; callers must not ignore it.
    pub error: Option<RegistryError<'a>>,
    /// Version at which this key last changed, not the global latest version.
    pub version: u64,
    /// Inline value; None distinguishes absent content from present empty bytes.
    pub value: Option<&'a [u8]>,
    /// Raw mutation timestamp in nanoseconds.
    pub timestamp_nanoseconds: u64,
}

/// Encode a Registry get-value request with an optional historical version.
///
/// None omits the version wrapper, asking for the latest value of the selected key.
/// # Errors
/// Returns `ByteLimit` before allocating if the encoded request exceeds `max_bytes`.
pub fn encode_get_value_request(
    key: &[u8],
    version: Option<u64>,
    max_bytes: usize,
) -> Result<Vec<u8>, Error> {
    let version_size = version.map_or(0, |v| 3 + varint_size(v));
    let size = key
        .len()
        .checked_add(1 + varint_size(key.len() as u64))
        .and_then(|n| n.checked_add(version_size))
        .ok_or(Error::ByteLimit)?;
    if size > max_bytes {
        return Err(Error::ByteLimit);
    }
    let mut output = Vec::with_capacity(size);
    if let Some(version) = version {
        output.extend_from_slice(&[10, (1 + varint_size(version)) as u8, 8]);
        write_varint(version, &mut output);
    }
    output.push(18);
    write_varint(key.len() as u64, &mut output);
    output.extend_from_slice(key);
    Ok(output)
}

/// Decode a bounded response without copying its blobs or error text.
/// # Errors
/// Refuses malformed/duplicate fields, unsupported wire types, chunked values,
/// invalid UTF-8 and exhausted byte/field budgets. Application errors are retained.
pub fn decode_get_value_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<GetValueResponse<'_>, Error> {
    admit_bytes(bytes, limits)?;
    let mut fields = limits.fields;
    let mut cursor = Cursor { bytes };
    let mut seen = 0;
    let mut result = GetValueResponse {
        error: None,
        version: 0,
        value: None,
        timestamp_nanoseconds: 0,
    };
    while let Some((tag, field)) = cursor.next(&mut fields)? {
        if (1..=5).contains(&tag) {
            singular(&mut seen, tag)?;
        }
        match tag {
            1 => result.error = Some(decode_error(field.blob()?, &mut fields)?),
            2 => result.version = field.integer()?,
            3 => result.value = Some(field.blob()?),
            4 => {
                field.blob()?;
                return Err(Error::ChunkedValue);
            }
            5 => result.timestamp_nanoseconds = field.integer()?,
            _ => {}
        }
    }
    Ok(result)
}

/// Decode subnet-list entry bytes, preserving source order and values.
///
/// Principal validity, duplicates, emptiness and report completeness belong to
/// consumers. The source bytes outlive the returned entry references.
/// # Errors
/// Refuses malformed fields and byte/field/entry/allocation limit violations.
/// Counts entries before allocating, so input lengths cannot allocate unchecked.
pub fn decode_subnet_list(bytes: &[u8], limits: Limits) -> Result<Vec<&[u8]>, Error> {
    admit_bytes(bytes, limits)?;
    let mut fields = limits.fields;
    let mut cursor = Cursor { bytes };
    let mut count = 0usize;
    while let Some((tag, field)) = cursor.next(&mut fields)? {
        if tag == 2 {
            field.blob()?;
            count = count.checked_add(1).ok_or(Error::ItemLimit)?;
            if count > limits.items {
                return Err(Error::ItemLimit);
            }
            if count > limits.allocation / std::mem::size_of::<&[u8]>() {
                return Err(Error::AllocationLimit);
            }
        }
    }
    let mut result = Vec::with_capacity(count);
    let mut fields = limits.fields;
    let mut cursor = Cursor { bytes };
    while let Some((tag, field)) = cursor.next(&mut fields)? {
        if tag == 2 {
            result.push(field.blob()?);
        }
    }
    Ok(result)
}

fn decode_error<'a>(bytes: &'a [u8], fields: &mut usize) -> Result<RegistryError<'a>, Error> {
    let mut cursor = Cursor { bytes };
    let mut seen = 0;
    let mut result = RegistryError {
        code: 0,
        reason: "",
        key: &[],
    };
    while let Some((tag, field)) = cursor.next(fields)? {
        if (1..=3).contains(&tag) {
            singular(&mut seen, tag)?;
        }
        match tag {
            // Protobuf int32 stores negative values as sign-extended varints.
            1 => result.code = field.integer()? as i32,
            2 => {
                result.reason = std::str::from_utf8(field.blob()?).map_err(|_| Error::Malformed)?
            }
            3 => result.key = field.blob()?,
            _ => {}
        }
    }
    Ok(result)
}

fn admit_bytes(bytes: &[u8], limits: Limits) -> Result<(), Error> {
    if bytes.len() > limits.bytes {
        Err(Error::ByteLimit)
    } else {
        Ok(())
    }
}

fn singular(seen: &mut u8, tag: u32) -> Result<(), Error> {
    let bit = 1 << tag;
    if *seen & bit != 0 {
        return Err(Error::DuplicateField);
    }
    *seen |= bit;
    Ok(())
}

fn varint_size(value: u64) -> usize {
    ((64 - value.leading_zeros()).max(1) as usize).div_ceil(7)
}

fn write_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 128 {
        output.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    output.push(value as u8);
}

enum Field<'a> {
    Integer(u64),
    Blob(&'a [u8]),
    Fixed,
}

impl<'a> Field<'a> {
    fn integer(self) -> Result<u64, Error> {
        match self {
            Self::Integer(value) => Ok(value),
            _ => Err(Error::Malformed),
        }
    }
    fn blob(self) -> Result<&'a [u8], Error> {
        match self {
            Self::Blob(value) => Ok(value),
            _ => Err(Error::Malformed),
        }
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn take(&mut self, size: usize) -> Result<&'a [u8], Error> {
        let (head, tail) = self.bytes.split_at_checked(size).ok_or(Error::Malformed)?;
        self.bytes = tail;
        Ok(head)
    }
    fn varint(&mut self) -> Result<u64, Error> {
        let mut value = 0;
        for index in 0..10 {
            let byte = self.take(1)?[0];
            if index == 9 && byte > 1 {
                return Err(Error::Malformed);
            }
            value |= u64::from(byte & 127) << (7 * index);
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(Error::Malformed)
    }
    fn next(&mut self, fields: &mut usize) -> Result<Option<(u32, Field<'a>)>, Error> {
        if self.bytes.is_empty() {
            return Ok(None);
        }
        *fields = fields.checked_sub(1).ok_or(Error::FieldLimit)?;
        let key = self.varint()?;
        let tag = u32::try_from(key >> 3).map_err(|_| Error::Malformed)?;
        if tag == 0 || tag > 0x1fff_ffff {
            return Err(Error::Malformed);
        }
        let field = match key & 7 {
            0 => Field::Integer(self.varint()?),
            1 => {
                self.take(8)?;
                Field::Fixed
            }
            2 => {
                let size = usize::try_from(self.varint()?).map_err(|_| Error::Malformed)?;
                Field::Blob(self.take(size)?)
            }
            5 => {
                self.take(4)?;
                Field::Fixed
            }
            _ => return Err(Error::UnsupportedWireType),
        };
        Ok(Some((tag, field)))
    }
}

#[cfg(test)]
mod tests;
