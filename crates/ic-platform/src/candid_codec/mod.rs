//! Owned binary Candid codec for explicit external-interface adapters.
//!
//! Trusted Rust implementations describe their schema separately from values.
//! Binary operations use no dynamic value tree, Serde visitor or text parser.
//! Select `candid-codec` to use this boundary without the registry codec. This
//! surface is not a drop-in replacement for `CandidType` or Serde.
//! Existing endpoints retain their qualified codec until separately migrated.
//! `references::Func` supplies typed function payloads using the owned Principal
//! and signature subtyping. Standalone typed service values are not yet supplied.
//! Custom mappings are trusted code and must use
//! the reader's traversal helpers to preserve admission accounting.
//!
//! ```
//! use ic_platform::candid_codec::{arguments::encode_args, decoder::{decode_args, Arity, Limits}};
//! let bytes = encode_args(&(42u64, "hello"), 1024).unwrap();
//! let limits = Limits { bytes: 1024, header: 128, types: 16, work: 4096, allocation: 4096 };
//! let result: (u64, String) = decode_args(&bytes, limits, Arity::Exact).unwrap();
//! assert_eq!(result, (42, "hello".to_owned()));
//! ```
pub mod arguments;
mod containers;
pub mod decoder;
#[cfg(feature = "candid-codec-endpoint")]
pub mod endpoint;
mod int;
mod integers;
mod mappings;
mod nat;
mod reserved;
pub use reserved::Reserved;
pub mod references;
mod schema;
use std::ptr;

/// Arbitrary-size Candid integer, backed by the maintained bigint implementation.
pub use int::Int;
/// Arbitrary-size Candid natural, backed by the maintained bigint implementation.
pub use nat::Nat;

/// Trusted static wire schema. Record/variant labels must be strictly increasing.
#[derive(Debug)]
pub enum Type {
    /// Resolve one stable schema anchor for a recursive trusted mapping.
    Deferred(fn() -> &'static Type),
    /// Candid primitive opcode (-1 through -17, or -24).
    Primitive(i8),
    /// Optional value.
    Opt(&'static Type),
    /// Homogeneous vector.
    Vec(&'static Type),
    /// Ordered numeric labels and field types.
    Record(&'static [(u32, &'static Type)]),
    /// Ordered numeric labels and alternative payload types.
    Variant(&'static [(u32, &'static Type)]),
    /// Function reference signature.
    Function {
        /// Argument sequence.
        args: &'static [&'static Type],
        /// Result sequence.
        results: &'static [&'static Type],
        /// Empty for update, or a single query/oneway/composite-query mode byte.
        annotations: &'static [u8],
    },
    /// Lexically ordered method names and function signatures.
    Service(&'static [(&'static str, &'static Type)]),
}

/// Encoding refusal; no partial frame is returned by the top-level functions.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid trusted schema, including unsorted or duplicate labels.
    Schema,
    /// Output or schema-table limit exceeded.
    Limit,
}

/// Bounded output sink for trusted typed mappings.
pub struct Writer {
    bytes: Vec<u8>,
    limit: usize,
}
impl Writer {
    /// Append bytes within the caller's output limit.
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(Error::Limit);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    /// Append an unsigned LEB128 integer.
    pub fn unsigned(&mut self, mut n: u128) -> Result<(), Error> {
        loop {
            let byte = (n & 127) as u8;
            n >>= 7;
            self.bytes(&[byte | if n == 0 { 0 } else { 128 }])?;
            if n == 0 {
                return Ok(());
            }
        }
    }
    /// Append a signed LEB128 integer.
    pub fn signed(&mut self, mut n: i128) -> Result<(), Error> {
        loop {
            let byte = (n & 127) as u8;
            n >>= 7;
            let done = (n == 0 && byte & 64 == 0) || (n == -1 && byte & 64 != 0);
            self.bytes(&[byte | if done { 0 } else { 128 }])?;
            if done {
                return Ok(());
            }
        }
    }
}

/// Trusted mapping from a Rust value to its Candid schema and payload.
pub trait Encode {
    /// Static schema; recursive definitions must reuse stable references.
    const TYPE: &'static Type;
    /// Write only the value payload, matching `TYPE` exactly.
    fn write(&self, output: &mut Writer) -> Result<(), Error>;
    /// Write consecutive element payloads, without a vector length prefix.
    /// Overrides must produce exactly the same bytes as calling `write` on each
    /// element and enforce the same output limit. The default supports custom mappings.
    fn write_slice(values: &[Self], output: &mut Writer) -> Result<(), Error>
    where
        Self: Sized,
    {
        for value in values {
            value.write(output)?;
        }
        Ok(())
    }
}

// Pointer identity terminates recursive schema traversal. Structurally equal
// declarations may occupy distinct entries; Candid permits that representation.
fn register(
    root: &'static Type,
    table: &mut Vec<&'static Type>,
    limit: usize,
) -> Result<(), Error> {
    // Iterative depth-first traversal keeps recursive and deeply nested trusted
    // schemas off the call stack while retaining the original table ordering.
    let mut pending = vec![root];
    while let Some(t) = pending.pop() {
        let t = t.resolved()?;
        if let Type::Primitive(code) = t {
            if !matches!(*code, -17..=-1 | -24) {
                return Err(Error::Schema);
            }
            continue;
        }
        if table.iter().any(|entry| ptr::eq(*entry, t)) {
            continue;
        }
        // Each table entry requires at least one output byte. This follows the
        // caller's output bound rather than imposing an unrelated type ceiling.
        if table.len() >= limit {
            return Err(Error::Limit);
        }
        table.push(t);
        match t {
            Type::Opt(inner) | Type::Vec(inner) => pending.push(inner),
            Type::Record(fields) | Type::Variant(fields) => {
                if fields.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
                    return Err(Error::Schema);
                }
                pending.extend(fields.iter().rev().map(|(_, field)| *field));
            }
            Type::Function {
                args,
                results,
                annotations,
            } => {
                if !matches!(*annotations, [] | [1] | [2] | [3])
                    || (*annotations == [2] && !results.is_empty())
                {
                    return Err(Error::Schema);
                }
                pending.extend(args.iter().chain(results.iter()).rev().copied());
            }
            Type::Service(methods) => {
                if methods.windows(2).any(|pair| pair[0].0 >= pair[1].0)
                    || methods
                        .iter()
                        .any(|(_, method)| !matches!(method.resolved(), Ok(Type::Function { .. })))
                {
                    return Err(Error::Schema);
                }
                pending.extend(methods.iter().rev().map(|(_, method)| *method));
            }
            Type::Primitive(_) | Type::Deferred(_) => unreachable!(),
        }
    }
    Ok(())
}
fn reference(
    t: &'static Type,
    aliases: &schema::Aliases,
    output: &mut Writer,
) -> Result<(), Error> {
    let t = t.resolved()?;
    let index = match t {
        Type::Primitive(code) => i128::from(*code),
        _ => aliases
            .iter()
            .find(|(source, _)| ptr::eq(*source, t))
            .map(|(_, index)| *index)
            .ok_or(Error::Schema)? as i128,
    };
    output.signed(index)
}
fn refs(
    types: &[&'static Type],
    aliases: &schema::Aliases,
    output: &mut Writer,
) -> Result<(), Error> {
    output.unsigned(types.len() as u128)?;
    for t in types {
        reference(t, aliases, output)?;
    }
    Ok(())
}
fn header(types: &[&'static Type], output: &mut Writer) -> Result<(), Error> {
    let mut table = Vec::new();
    for t in types {
        register(t, &mut table, output.limit)?;
    }
    let aliases = schema::compact(&mut table)?;
    output.bytes(b"DIDL")?;
    output.unsigned(table.len() as u128)?;
    for t in &table {
        match t {
            Type::Opt(inner) | Type::Vec(inner) => {
                output.signed(if matches!(t, Type::Opt(_)) { -18 } else { -19 })?;
                reference(inner, &aliases, output)?;
            }
            Type::Record(fields) | Type::Variant(fields) => {
                output.signed(if matches!(t, Type::Record(_)) {
                    -20
                } else {
                    -21
                })?;
                output.unsigned(fields.len() as u128)?;
                for (id, field) in *fields {
                    output.unsigned(u128::from(*id))?;
                    reference(field, &aliases, output)?;
                }
            }
            Type::Function {
                args,
                results,
                annotations,
            } => {
                output.signed(-22)?;
                refs(args, &aliases, output)?;
                refs(results, &aliases, output)?;
                output.unsigned(annotations.len() as u128)?;
                output.bytes(annotations)?;
            }
            Type::Service(methods) => {
                output.signed(-23)?;
                output.unsigned(methods.len() as u128)?;
                for (name, method) in *methods {
                    name.write(output)?;
                    reference(method, &aliases, output)?;
                }
            }
            Type::Primitive(_) | Type::Deferred(_) => unreachable!(),
        }
    }
    refs(types, &aliases, output)
}

/// Encode exactly one argument, with at most `limit` output bytes.
pub fn encode_one<T: Encode + ?Sized>(value: &T, limit: usize) -> Result<Vec<u8>, Error> {
    arguments::encode_args(&(value,), limit)
}

impl<T: Encode + ?Sized> Encode for &T {
    const TYPE: &'static Type = T::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        T::write(self, output)
    }
}

macro_rules! fixed {
    ($($t:ty => $code:expr),* $(,)?) => {$ (
        impl Encode for $t {
            const TYPE: &'static Type = &Type::Primitive($code);
            fn write(&self, output: &mut Writer) -> Result<(), Error> { output.bytes(&self.to_le_bytes()) }
        }
    )*};
}
fixed!(u16 => -6, u32 => -7, u64 => -8, i8 => -9, i16 => -10, i32 => -11, i64 => -12, f32 => -13, f64 => -14);
impl Encode for u8 {
    const TYPE: &'static Type = &Type::Primitive(-5);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        output.bytes(&[*self])
    }
    fn write_slice(values: &[Self], output: &mut Writer) -> Result<(), Error> {
        output.bytes(values)
    }
}
impl Encode for bool {
    const TYPE: &'static Type = &Type::Primitive(-2);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        output.bytes(&[u8::from(*self)])
    }
}
impl Encode for () {
    const TYPE: &'static Type = &Type::Primitive(-1);
    fn write(&self, _: &mut Writer) -> Result<(), Error> {
        Ok(())
    }
}
impl Encode for str {
    const TYPE: &'static Type = &Type::Primitive(-15);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        output.unsigned(self.len() as u128)?;
        output.bytes(self.as_bytes())
    }
}
impl Encode for String {
    const TYPE: &'static Type = str::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        self.as_str().write(output)
    }
}
impl<T: Encode> Encode for Option<T> {
    const TYPE: &'static Type = &Type::Opt(T::TYPE);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        output.bytes(&[u8::from(self.is_some())])?;
        if let Some(value) = self {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<T: Encode> Encode for Vec<T> {
    const TYPE: &'static Type = &Type::Vec(T::TYPE);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        self.as_slice().write(output)
    }
}
impl<A: Encode, B: Encode> Encode for (A, B) {
    const TYPE: &'static Type = &Type::Record(&[(0, A::TYPE), (1, B::TYPE)]);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        self.0.write(output)?;
        self.1.write(output)
    }
}

#[cfg(test)]
mod tests;

impl<T: Encode, E: Encode> Encode for Result<T, E> {
    // Candid field identifiers for "Ok" and "Err", in numeric order.
    const TYPE: &'static Type = &Type::Variant(&[(17724, T::TYPE), (3456837, E::TYPE)]);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        match self {
            Ok(value) => {
                output.unsigned(0)?;
                value.write(output)
            }
            Err(value) => {
                output.unsigned(1)?;
                value.write(output)
            }
        }
    }
}

#[cfg(feature = "principal")]
impl Encode for crate::Principal {
    const TYPE: &'static Type = &Type::Primitive(-24);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        output.bytes(&[1])?;
        output.unsigned(self.as_slice().len() as u128)?;
        output.bytes(self.as_slice())
    }
}

impl Type {
    /// Resolve a trusted recursive anchor. Chained or self-returning anchors are invalid.
    /// The resolver must always return the same static schema.
    pub fn resolved(&'static self) -> Result<&'static Self, Error> {
        let schema = match self {
            Self::Deferred(resolve) => resolve(),
            other => other,
        };
        if matches!(schema, Self::Deferred(_)) {
            Err(Error::Schema)
        } else {
            Ok(schema)
        }
    }
}
