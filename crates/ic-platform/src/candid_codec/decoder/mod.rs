//! Bounded typed binary reader with validated skipping and a 16-level wire ceiling.
//! Limits are explicit and have no defaults. Work units and cumulative allocation
//! charges are specific to this codec, not interchangeable with upstream quotas.
use super::Encode;
mod containers;
mod integers;
mod subtyping;

/// Caller-selected admission budgets. These do not bound arbitrary custom mappings.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum complete input length in bytes.
    pub bytes: usize,
    /// Maximum byte offset at the end of the header and argument type list.
    pub header: usize,
    /// Maximum constructed entries in the wire type table.
    pub types: usize,
    /// Cumulative byte reads and traversal charges, including failed optional attempts.
    pub work: usize,
    /// Cumulative logical storage charges; not an allocator-level peak-heap bound.
    pub allocation: usize,
}
/// Closed decoding refusals. Malformed input and exhausted budgets never become `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid framing, references, value bytes or truncated input.
    Malformed,
    /// Unsupported constructor or opaque reference form.
    Unsupported,
    /// The incoming value cannot map to the requested Rust type.
    Type,
    /// Exact argument count differs from the requested sequence.
    Arity,
    /// Bytes remain after all supplied arguments have been validated.
    Trailing,
    /// A caller budget or the fixed 16-level wire admission ceiling was exceeded.
    Limit,
}
type Result<T> = std::result::Result<T, Error>;
type Id = i64;
#[derive(Debug)]
enum Wire {
    Opt(Id),
    Vec(Id),
    Record(Vec<(u32, Id)>),
    Variant(Vec<(u32, Id)>),
    Function(Vec<Id>, Vec<Id>, u8),
    Service(Vec<(String, Id)>),
}

/// Validated message reader passed to trusted typed mappings.
pub struct Reader<'a> {
    input: &'a [u8],
    pos: usize,
    work: usize,
    skipping: usize,
    in_skip: bool,
    allocation: usize,
    table: Vec<Wire>,
    depth: usize,
    header: Option<usize>,
}
impl<'a> Reader<'a> {
    fn charge(&mut self, count: usize) -> Result<()> {
        self.work = self.work.checked_sub(count).ok_or(Error::Limit)?;
        if self.in_skip {
            self.skipping = self.skipping.checked_sub(count).ok_or(Error::Limit)?;
        }
        Ok(())
    }
    fn allocate(&mut self, count: usize) -> Result<()> {
        self.allocation = self.allocation.checked_sub(count).ok_or(Error::Limit)?;
        Ok(())
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        self.charge(count)?;
        let end = self.pos.checked_add(count).ok_or(Error::Limit)?;
        if self.header.is_some_and(|limit| end > limit) {
            return Err(Error::Limit);
        }
        let bytes = self.input.get(self.pos..end).ok_or(Error::Malformed)?;
        self.pos = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn unsigned(&mut self) -> Result<u64> {
        let mut result = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.byte()?;
            if shift == 63 && byte & 127 > 1 {
                return Err(Error::Malformed);
            }
            result |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                return Ok(result);
            }
        }
        Err(Error::Malformed)
    }
    fn signed(&mut self) -> Result<i64> {
        let mut result = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.byte()?;
            if shift == 63 && !matches!(byte & 127, 0 | 127) {
                return Err(Error::Malformed);
            }
            result |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                if shift < 63 && byte & 64 != 0 {
                    result |= u64::MAX << (shift + 7);
                }
                return Ok(result as i64);
            }
        }
        Err(Error::Malformed)
    }
    fn count(&mut self) -> Result<usize> {
        usize::try_from(self.unsigned()?).map_err(|_| Error::Limit)
    }
    fn text(&mut self) -> Result<&'a str> {
        let n = self.count()?;
        std::str::from_utf8(self.take(n)?).map_err(|_| Error::Malformed)
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Malformed),
        }
    }
    fn ids(&mut self) -> Result<Vec<Id>> {
        let n = self.count()?;
        self.charge(n)?;
        self.allocate(n.checked_mul(size_of::<Id>()).ok_or(Error::Limit)?)?;
        let mut values = Vec::new();
        values.try_reserve_exact(n).map_err(|_| Error::Limit)?;
        for _ in 0..n {
            values.push(self.signed()?);
        }
        Ok(values)
    }
    fn fields(&mut self) -> Result<Vec<(u32, Id)>> {
        let n = self.count()?;
        self.charge(n)?;
        self.allocate(n.checked_mul(size_of::<(u32, Id)>()).ok_or(Error::Limit)?)?;
        let mut fields = Vec::new();
        fields.try_reserve_exact(n).map_err(|_| Error::Limit)?;
        for _ in 0..n {
            let id = u32::try_from(self.unsigned()?).map_err(|_| Error::Malformed)?;
            if fields.last().is_some_and(|(last, _)| *last >= id) {
                return Err(Error::Malformed);
            }
            fields.push((id, self.signed()?));
        }
        Ok(fields)
    }
    fn valid_id(&self, id: Id) -> Result<()> {
        if matches!(id, -17..=-1 | -24) || (id >= 0 && (id as u64) < self.table.len() as u64) {
            Ok(())
        } else {
            Err(Error::Malformed)
        }
    }
    fn node(&self, id: Id) -> Result<&Wire> {
        self.table
            .get(usize::try_from(id).map_err(|_| Error::Type)?)
            .ok_or(Error::Malformed)
    }
    fn type_depth(&mut self, id: Id, path: &mut [Id; 16], depth: usize) -> Result<()> {
        self.charge(1)?;
        if id < 0 || path[..depth].contains(&id) {
            return Ok(());
        }
        if depth == path.len() {
            return Err(Error::Limit);
        }
        path[depth] = id;
        let count = match self.node(id)? {
            Wire::Opt(_) | Wire::Vec(_) => 1,
            Wire::Record(fields) | Wire::Variant(fields) => fields.len(),
            Wire::Function(args, results, mode) => {
                debug_assert!(*mode <= 3, "header admission validates function modes");
                args.len() + results.len()
            }
            Wire::Service(methods) => methods.len(),
        };
        for index in 0..count {
            let child = match self.node(id)? {
                Wire::Opt(child) | Wire::Vec(child) => *child,
                Wire::Record(fields) | Wire::Variant(fields) => fields[index].1,
                Wire::Function(args, results, _) => {
                    if index < args.len() {
                        args[index]
                    } else {
                        results[index - args.len()]
                    }
                }
                Wire::Service(methods) => methods[index].1,
            };
            self.type_depth(child, path, depth + 1)?;
        }
        Ok(())
    }
    fn enter<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.charge(1)?;
        if self.depth == 16 {
            return Err(Error::Limit);
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }
    /// Decode a field using its supplied wire type and charge traversal work.
    pub fn value<T: Decode<'a>>(&mut self, id: Id) -> Result<T> {
        self.charge(1)?;
        T::read(self, id)
    }
    /// Decode a vector with a caller-selected element cap, checked before reserving
    /// its storage or decoding any element. Nested fields can select different caps
    /// through their own `Decode` mappings. Global work/allocation limits still apply.
    pub fn bounded_vec<T: Decode<'a>>(&mut self, id: Id, maximum: usize) -> Result<Vec<T>> {
        self.enter(|r| {
            let Wire::Vec(inner) = r.node(id)? else {
                return Err(Error::Type);
            };
            let inner = *inner;
            let n = r.count()?;
            if n > maximum {
                return Err(Error::Limit);
            }
            r.charge(n)?;
            r.allocate(n.checked_mul(size_of::<T>()).ok_or(Error::Limit)?)?;
            let mut values = Vec::new();
            values.try_reserve_exact(n).map_err(|_| Error::Limit)?;
            for _ in 0..n {
                values.push(r.value::<T>(inner)?);
            }
            Ok(values)
        })
    }
    /// Visit record fields, validating and skipping any callback returns `false` for.
    /// Set `tuple` only when numeric labels must be contiguous from zero.
    pub fn record(
        &mut self,
        id: Id,
        tuple: bool,
        mut field: impl FnMut(u32, Id, &mut Self) -> Result<bool>,
    ) -> Result<()> {
        self.enter(|r| {
            let Wire::Record(fields) = r.node(id)? else {
                return Err(Error::Type);
            };
            if tuple
                && fields
                    .iter()
                    .enumerate()
                    .any(|(index, field)| u64::from(field.0) != index as u64)
            {
                return Err(Error::Type);
            }
            let count = fields.len();
            for index in 0..count {
                let Wire::Record(fields) = r.node(id)? else {
                    unreachable!()
                };
                let (label, inner) = fields[index];
                if !field(label, inner, r)? {
                    r.skip(inner)?;
                }
            }
            Ok(())
        })
    }
    /// Decode the selected variant through a callback receiving its numeric label.
    pub fn variant<T>(
        &mut self,
        id: Id,
        select: impl FnOnce(u32, Id, &mut Self) -> Result<T>,
    ) -> Result<T> {
        self.enter(|r| {
            // Check the wire shape before consuming a variant index: a shape
            // mismatch is recoverable by an enclosing optional conversion.
            if !matches!(r.node(id)?, Wire::Variant(_)) {
                return Err(Error::Type);
            }
            let index = r.count()?;
            let Wire::Variant(fields) = r.node(id)? else {
                unreachable!()
            };
            let (label, inner) = fields.get(index).copied().ok_or(Error::Malformed)?;
            select(label, inner, r)
        })
    }
    /// Read an exact wire option without bare-value promotion or incompatible-value
    /// fallback. An absent record field is handled separately by `Decode::missing`.
    pub fn strict_option<T: Decode<'a>>(&mut self, id: Id) -> Result<Option<T>> {
        let Wire::Opt(inner) = self.node(id)? else {
            return Err(Error::Type);
        };
        let inner = *inner;
        self.enter(|r| {
            if r.boolean()? {
                r.value(inner).map(Some)
            } else {
                Ok(None)
            }
        })
    }
    fn optional<T: Decode<'a>>(&mut self, id: Id) -> Result<Option<T>> {
        let position = self.pos;
        match self.value::<T>(id) {
            Ok(value) => Ok(Some(value)),
            Err(Error::Type) => {
                // Retain all work/allocation charges from the failed attempt.
                // Only the cursor rolls back before validating the skipped value.
                self.pos = position;
                self.skip(id)?;
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
    fn principal_bytes(&mut self) -> Result<&'a [u8]> {
        if self.byte()? != 1 {
            return Err(Error::Unsupported);
        }
        let count = self.count()?;
        if count > 29 {
            return Err(Error::Malformed);
        }
        self.take(count)
    }
    fn principal(&mut self) -> Result<()> {
        self.principal_bytes().map(|_| ())
    }
    /// Validate and discard a value with the same wire depth and work accounting.
    pub fn skip(&mut self, id: Id) -> Result<()> {
        let previous = self.in_skip;
        self.in_skip = true;
        let result = self.skip_value(id);
        self.in_skip = previous;
        result
    }
    fn skip_value(&mut self, id: Id) -> Result<()> {
        self.charge(1)?;
        match id {
            -1 | -16 => Ok(()),
            -2 => {
                self.boolean()?;
                Ok(())
            }
            -3 | -4 => {
                while self.byte()? & 128 != 0 {}
                Ok(())
            }
            -5 | -9 => {
                self.take(1)?;
                Ok(())
            }
            -6 | -10 => {
                self.take(2)?;
                Ok(())
            }
            -7 | -11 | -13 => {
                self.take(4)?;
                Ok(())
            }
            -8 | -12 | -14 => {
                self.take(8)?;
                Ok(())
            }
            -15 => {
                self.text()?;
                Ok(())
            }
            -17 => Err(Error::Malformed),
            -24 => self.principal(),
            _ => self.enter(|reader| {
                match reader.node(id)? {
                    Wire::Opt(inner) => {
                        let inner = *inner;
                        if reader.boolean()? {
                            reader.skip(inner)?;
                        }
                    }
                    Wire::Vec(inner) => {
                        let inner = *inner;
                        let count = reader.count()?;
                        reader.charge(count)?;
                        for _ in 0..count {
                            reader.skip(inner)?;
                        }
                    }
                    Wire::Record(fields) => {
                        let count = fields.len();
                        for index in 0..count {
                            let Wire::Record(fields) = reader.node(id)? else {
                                unreachable!()
                            };
                            let inner = fields[index].1;
                            reader.skip(inner)?;
                        }
                    }
                    Wire::Variant(_) => {
                        let index = reader.count()?;
                        let Wire::Variant(fields) = reader.node(id)? else {
                            unreachable!()
                        };
                        let inner = fields.get(index).ok_or(Error::Malformed)?.1;
                        reader.skip(inner)?;
                    }
                    Wire::Function(..) => {
                        if reader.byte()? != 1 {
                            return Err(Error::Unsupported);
                        }
                        reader.principal()?;
                        reader.text()?;
                    }
                    Wire::Service(_) => reader.principal()?,
                }
                Ok(())
            }),
        }
    }
}

/// Trusted Rust mapping. Custom implementations must preserve reader accounting.
pub trait Decode<'a>: Encode + Sized {
    /// Read a value of the supplied wire type.
    fn read(reader: &mut Reader<'a>, wire: Id) -> Result<Self>;
    /// Resolve an absent field/argument; required values refuse by default.
    fn missing() -> Result<Self> {
        Err(Error::Type)
    }
}

fn message(bytes: &[u8], limits: Limits) -> Result<(Reader<'_>, Vec<Id>)> {
    if bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    let mut r = Reader {
        input: bytes,
        pos: 0,
        work: limits.work,
        skipping: limits.work,
        in_skip: false,
        allocation: limits.allocation,
        table: Vec::new(),
        depth: 0,
        header: Some(limits.header),
    };
    if r.take(4)? != b"DIDL" {
        return Err(Error::Malformed);
    }
    let count = r.count()?;
    if count > limits.types {
        return Err(Error::Limit);
    }
    r.charge(count)?;
    r.allocate(count.checked_mul(size_of::<Wire>()).ok_or(Error::Limit)?)?;
    r.table.try_reserve_exact(count).map_err(|_| Error::Limit)?;
    for _ in 0..count {
        let node = match r.signed()? {
            -18 => Wire::Opt(r.signed()?),
            -19 => Wire::Vec(r.signed()?),
            -20 => Wire::Record(r.fields()?),
            -21 => Wire::Variant(r.fields()?),
            -22 => {
                let args = r.ids()?;
                let results = r.ids()?;
                let count = r.count()?;
                if count > 1 {
                    return Err(Error::Malformed);
                }
                let modes = r.take(count)?.to_vec();
                r.allocate(count)?;
                if !matches!(modes.as_slice(), [] | [1] | [2] | [3])
                    || (modes == [2] && !results.is_empty())
                {
                    return Err(Error::Malformed);
                }
                Wire::Function(args, results, modes.first().copied().unwrap_or(0))
            }
            -23 => {
                let count = r.count()?;
                r.charge(count)?;
                r.allocate(
                    count
                        .checked_mul(size_of::<(String, Id)>())
                        .ok_or(Error::Limit)?,
                )?;
                let mut methods: Vec<(String, Id)> = Vec::new();
                methods.try_reserve_exact(count).map_err(|_| Error::Limit)?;
                for _ in 0..count {
                    let name = r.text()?;
                    r.allocate(name.len())?;
                    if methods
                        .last()
                        .is_some_and(|(previous, _)| previous.as_str() >= name)
                    {
                        return Err(Error::Malformed);
                    }
                    methods.push((name.to_owned(), r.signed()?));
                }
                Wire::Service(methods)
            }
            _ => return Err(Error::Unsupported),
        };
        r.table.push(node);
    }
    // Validate every reference, including unused table entries, before values.
    for node in &r.table {
        match node {
            Wire::Opt(id) | Wire::Vec(id) => r.valid_id(*id)?,
            Wire::Record(fields) | Wire::Variant(fields) => {
                for (_, id) in fields {
                    r.valid_id(*id)?;
                }
            }
            Wire::Function(args, results, _) => {
                for id in args.iter().chain(results) {
                    r.valid_id(*id)?;
                }
            }
            Wire::Service(methods) => {
                for (_, id) in methods {
                    r.valid_id(*id)?;
                    if !matches!(r.node(*id)?, Wire::Function(..)) {
                        return Err(Error::Malformed);
                    }
                }
            }
        }
    }
    for index in 0..r.table.len() {
        r.type_depth(index as Id, &mut [0; 16], 0)?;
    }
    let arguments = r.ids()?;
    for id in &arguments {
        r.valid_id(*id)?;
    }
    r.header = None;
    Ok((r, arguments))
}

/// Explicit argument-count policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arity {
    /// Require exactly the requested number of arguments.
    Exact,
    /// Allow typed missing values and validate/skip surplus arguments.
    Compatible,
}

/// Decode an argument sequence, validating the complete message and all unused types.
pub fn decode_args<'a, T: super::arguments::DecodeArgs<'a>>(
    bytes: &'a [u8],
    limits: Limits,
    arity: Arity,
) -> Result<T> {
    decode_args_with_skip_limit(bytes, limits, arity, limits.work)
}

/// Decode with an independent cumulative skipped-work budget, in addition to
/// `Limits::work`. Every byte-read and traversal charge made inside `Reader::skip`
/// consumes both budgets, including nested skips and optional fallback. Failed
/// attempts never refund charges. Zero refuses even zero-byte skipped values.
/// These units are codec-specific, not upstream Candid skipping quotas.
pub fn decode_args_with_skip_limit<'a, T: super::arguments::DecodeArgs<'a>>(
    bytes: &'a [u8],
    limits: Limits,
    arity: Arity,
    skipping: usize,
) -> Result<T> {
    let (mut r, arguments) = message(bytes, limits)?;
    r.skipping = skipping;
    if arity == Arity::Exact && arguments.len() != T::COUNT {
        return Err(Error::Arity);
    }
    let value = T::read_args(&mut r, &arguments)?;
    for id in arguments.iter().skip(T::COUNT) {
        r.skip(*id)?;
    }
    if r.pos != bytes.len() {
        return Err(Error::Trailing);
    }
    Ok(value)
}

/// Decode exactly one argument, refusing trailing bytes or surplus arguments.
pub fn decode_one<'a, T: Decode<'a>>(bytes: &'a [u8], limits: Limits) -> Result<T> {
    Ok(decode_args::<(T,)>(bytes, limits, Arity::Exact)?.0)
}

/// Decode exactly one argument with the separate skipped-work budget described
/// by [`decode_args_with_skip_limit`].
pub fn decode_one_with_skip_limit<'a, T: Decode<'a>>(
    bytes: &'a [u8],
    limits: Limits,
    skipping: usize,
) -> Result<T> {
    Ok(decode_args_with_skip_limit::<(T,)>(bytes, limits, Arity::Exact, skipping)?.0)
}

macro_rules! fixed {
    ($($t:ty => $code:expr),* $(,)?) => {$ (
        impl<'a> Decode<'a> for $t {
            fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
                if id != $code { return Err(Error::Type); }
                Ok(Self::from_le_bytes(r.take(size_of::<Self>())?.try_into().map_err(|_| Error::Malformed)?))
            }
        }
    )*};
}
fixed!(u8 => -5, u16 => -6, u32 => -7, u64 => -8, i8 => -9, i16 => -10, i32 => -11, i64 => -12, f32 => -13, f64 => -14);
impl<'a> Decode<'a> for bool {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        if id != -2 {
            return Err(Error::Type);
        }
        r.boolean()
    }
}
impl<'a> Decode<'a> for () {
    fn missing() -> Result<Self> {
        Ok(())
    }
    fn read(_: &mut Reader<'a>, id: Id) -> Result<Self> {
        if id != -1 { Err(Error::Type) } else { Ok(()) }
    }
}
impl<'a> Decode<'a> for String {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        if id != -15 {
            return Err(Error::Type);
        }
        let text = r.text()?;
        r.allocate(text.len())?;
        Ok(text.to_owned())
    }
}

#[cfg(feature = "principal")]
impl<'a> Decode<'a> for crate::Principal {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        if id != -24 {
            return Err(Error::Type);
        }
        Self::try_from_slice(r.principal_bytes()?).map_err(|_| Error::Malformed)
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Vec<T> {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        r.bounded_vec(id, usize::MAX)
    }
}
impl<'a, T: Decode<'a>> Decode<'a> for Option<T> {
    fn missing() -> Result<Self> {
        Ok(None)
    }
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        if matches!(id, -1 | -16) {
            r.skip(id)?;
            return Ok(None);
        }
        if let Ok(Wire::Opt(inner)) = r.node(id) {
            let inner = *inner;
            return r.enter(|r| {
                if !r.boolean()? {
                    return Ok(None);
                }
                r.optional::<T>(inner)
            });
        }
        // Preserve upstream nested promotion, including nat64 -> opt opt nat64.
        r.optional::<T>(id)
    }
}
impl<'a, A: Decode<'a>, B: Decode<'a>> Decode<'a> for (A, B) {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        let (mut a, mut b) = (None, None);
        r.record(id, true, |label, inner, r| {
            match label {
                0 => a = Some(r.value::<A>(inner)?),
                1 => b = Some(r.value::<B>(inner)?),
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok((
            match a {
                Some(v) => v,
                None => A::missing()?,
            },
            match b {
                Some(v) => v,
                None => B::missing()?,
            },
        ))
    }
}

#[cfg(test)]
mod tests;

impl<'a, T: Decode<'a>, E: Decode<'a>> Decode<'a> for std::result::Result<T, E> {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        r.variant(id, |label, inner, r| match label {
            17724 => Ok(Ok(r.value::<T>(inner)?)),
            3456837 => Ok(Err(r.value::<E>(inner)?)),
            _ => Err(Error::Type),
        })
    }
}
