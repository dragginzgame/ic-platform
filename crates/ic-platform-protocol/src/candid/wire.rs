use super::{Error, Limits};
use candidate_types::Principal;
use std::mem::size_of;

type Result<T> = std::result::Result<T, Error>;
type Id = i64;

// The table retains structure only. Service names borrow validated input text.
#[derive(Debug)]
enum Node<'a> {
    Option(Id),
    Vector(Id),
    Record(Vec<(u32, Id)>),
    Variant(Vec<(u32, Id)>),
    Function {
        arguments: Vec<Id>,
        results: Vec<Id>,
    },
    Service(Vec<(&'a str, Id)>),
}

struct Input<'a> {
    bytes: &'a [u8],
    offset: usize,
    limits: Limits,
    header: bool,
    skipping: bool,
}

impl<'a> Input<'a> {
    fn charge(&mut self, work: usize) -> Result<()> {
        self.limits.work = self.limits.work.checked_sub(work).ok_or(Error::Limit)?;
        if self.skipping {
            self.limits.skipping = self.limits.skipping.checked_sub(work).ok_or(Error::Limit)?;
        }
        Ok(())
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(length).ok_or(Error::Limit)?;
        if self.header && end > self.limits.header {
            return Err(Error::Limit);
        }
        self.charge(length)?;
        let value = self.bytes.get(self.offset..end).ok_or(Error::Malformed)?;
        self.offset = end;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    // Bounded header/count integers permit non-minimal encodings within 64 bits.
    fn unsigned(&mut self) -> Result<u64> {
        let mut result = 0;
        for index in 0..10 {
            let byte = self.byte()?;
            let payload = byte & 127;
            if index == 9 && payload > 1 {
                return Err(Error::Malformed);
            }
            result |= u64::from(payload) << (7 * index);
            if byte < 128 {
                return Ok(result);
            }
        }
        Err(Error::Malformed)
    }

    fn signed(&mut self) -> Result<i64> {
        let mut result = 0u64;
        for index in 0..10 {
            let byte = self.byte()?;
            let payload = byte & 127;
            if index == 9 && payload != 0 && payload != 127 {
                return Err(Error::Malformed);
            }
            result |= u64::from(payload) << (7 * index);
            if byte < 128 {
                let used = 7 * (index + 1);
                if used < 64 && byte & 64 != 0 {
                    result |= u64::MAX << used;
                }
                return Ok(result as i64);
            }
        }
        Err(Error::Malformed)
    }

    fn count(&mut self) -> Result<usize> {
        usize::try_from(self.unsigned()?).map_err(|_| Error::Limit)
    }

    fn storage(&mut self, bytes: usize) -> Result<()> {
        self.limits.allocation = self
            .limits
            .allocation
            .checked_sub(bytes)
            .ok_or(Error::Limit)?;
        Ok(())
    }

    fn reserve<T>(&mut self, count: usize) -> Result<Vec<T>> {
        self.storage(count.checked_mul(size_of::<T>()).ok_or(Error::Limit)?)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        Ok(values)
    }

    fn text(&mut self) -> Result<&'a str> {
        let count = self.count()?;
        std::str::from_utf8(self.take(count)?).map_err(|_| Error::Malformed)
    }

    fn principal(&mut self) -> Result<Principal> {
        if self.byte()? != 1 {
            return Err(Error::Unsupported);
        }
        let count = self.count()?;
        if count > Principal::MAX_LENGTH_IN_BYTES {
            return Err(Error::Malformed);
        }
        Principal::try_from_slice(self.take(count)?).map_err(|_| Error::Malformed)
    }

    fn boolean(&mut self) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Malformed),
        }
    }
}

fn ids(input: &mut Input<'_>) -> Result<Vec<Id>> {
    let count = input.count()?;
    if count > u32::MAX as usize {
        return Err(Error::Malformed);
    }
    // Every reference requires at least one byte and a traversal operation.
    input.charge(count)?;
    let mut values = input.reserve(count)?;
    for _ in 0..count {
        values.push(input.signed()?);
    }
    Ok(values)
}

fn header<'a>(input: &mut Input<'a>, empty: bool) -> Result<(Vec<Node<'a>>, Id)> {
    if input.take(4)? != b"DIDL" {
        return Err(Error::Malformed);
    }
    let count = input.count()?;
    if count > input.limits.types {
        return Err(Error::Limit);
    }
    input.charge(count)?;
    let mut nodes = input.reserve(count)?;
    for _ in 0..count {
        let node = match input.signed()? {
            -18 => Node::Option(input.signed()?),
            -19 => Node::Vector(input.signed()?),
            opcode @ (-20 | -21) => {
                let count = input.count()?;
                input.charge(count)?;
                let mut fields = input.reserve(count)?;
                let mut previous = None;
                for _ in 0..count {
                    let label = u32::try_from(input.unsigned()?).map_err(|_| Error::Malformed)?;
                    if previous.is_some_and(|old| old >= label) {
                        return Err(Error::Malformed);
                    }
                    previous = Some(label);
                    fields.push((label, input.signed()?));
                }
                if opcode == -20 {
                    Node::Record(fields)
                } else {
                    Node::Variant(fields)
                }
            }
            -22 => {
                let arguments = ids(input)?;
                let results = ids(input)?;
                let count = input.count()?;
                if count > 1 {
                    return Err(Error::Malformed);
                }
                if count == 1 {
                    match input.byte()? {
                        1 | 3 => (),
                        2 if results.is_empty() => (),
                        _ => return Err(Error::Malformed),
                    }
                }
                Node::Function { arguments, results }
            }
            -23 => {
                let count = input.count()?;
                input.charge(count)?;
                let mut methods = input.reserve(count)?;
                let mut previous: Option<&str> = None;
                for _ in 0..count {
                    let name = input.text()?;
                    if previous.is_some_and(|old| old >= name) {
                        return Err(Error::Malformed);
                    }
                    previous = Some(name);
                    methods.push((name, input.signed()?));
                }
                Node::Service(methods)
            }
            _ => return Err(Error::Unsupported),
        };
        nodes.push(node);
    }
    if input.count()? != usize::from(!empty) {
        return Err(Error::Arity);
    }
    let root = if empty { -1 } else { input.signed()? };
    valid_id(root, &nodes)?;
    // Validate even unused definitions, including function/service references.
    for node in &nodes {
        match node {
            Node::Option(id) | Node::Vector(id) => valid_id(*id, &nodes)?,
            Node::Record(fields) | Node::Variant(fields) => {
                for (_, id) in fields {
                    input.charge(1)?;
                    valid_id(*id, &nodes)?;
                }
            }
            Node::Function { arguments, results } => {
                for id in arguments.iter().chain(results) {
                    input.charge(1)?;
                    valid_id(*id, &nodes)?;
                }
            }
            Node::Service(methods) => {
                for (_, id) in methods {
                    input.charge(1)?;
                    if !matches!(node_at(*id, &nodes)?, Node::Function { .. }) {
                        return Err(Error::Malformed);
                    }
                }
            }
        }
    }
    // Cycles are legal, but any simple constructed path has at most 16 nodes.
    for index in 0..nodes.len() {
        depth(index as Id, &nodes, input, &mut [0; 16], 0)?;
    }
    input.header = false;
    Ok((nodes, root))
}

fn valid_id(id: Id, nodes: &[Node<'_>]) -> Result<()> {
    if (-17..=-1).contains(&id) || id == -24 {
        return Ok(());
    }
    node_at(id, nodes).map(|_| ())
}

fn node_at<'n, 'a>(id: Id, nodes: &'n [Node<'a>]) -> Result<&'n Node<'a>> {
    usize::try_from(id)
        .ok()
        .and_then(|index| nodes.get(index))
        .ok_or(Error::Malformed)
}

fn depth(
    id: Id,
    nodes: &[Node<'_>],
    input: &mut Input<'_>,
    path: &mut [Id; 16],
    used: usize,
) -> Result<()> {
    input.charge(1)?;
    if id < 0 || path[..used].contains(&id) {
        return Ok(());
    }
    if used == 16 {
        return Err(Error::Limit);
    }
    path[used] = id;
    match node_at(id, nodes)? {
        Node::Option(child) | Node::Vector(child) => depth(*child, nodes, input, path, used + 1)?,
        Node::Record(fields) | Node::Variant(fields) => {
            for (_, child) in fields {
                depth(*child, nodes, input, path, used + 1)?;
            }
        }
        Node::Function { arguments, results } => {
            for child in arguments.iter().chain(results) {
                depth(*child, nodes, input, path, used + 1)?;
            }
        }
        Node::Service(methods) => {
            for (_, child) in methods {
                depth(*child, nodes, input, path, used + 1)?;
            }
        }
    }
    Ok(())
}

// Decode has no encoding supertrait. The schema is generated by the same
// declaration as its Rust fields and reader; only required/strict optional
// selected fields are supported in this initial projection API.
#[derive(PartialEq, Eq)]
pub(crate) enum Schema {
    Primitive(Id),
    Vector(&'static Schema),
    StrictOption(&'static Schema),
    Record(&'static [(u32, &'static Schema)]),
    #[cfg(any(
        feature = "cloud-engine-marketplace",
        feature = "registry-placement",
        feature = "management-snapshots",
        feature = "caffeine",
        feature = "sns-neurons"
    ))]
    Variant(&'static [(u32, &'static Schema)]),
}
pub(crate) trait Decode: Sized {
    const SCHEMA: Schema;
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self>;
}

pub(crate) struct Reader<'n, 'a> {
    table: &'n [Node<'a>],
    input: Input<'a>,
    depth: usize,
    #[cfg(feature = "management-snapshots")]
    next_vector_limit: Option<usize>,
}

impl Reader<'_, '_> {
    // Limit the selected container, including through a present option, without
    // accidentally applying its item-count cap to vectors inside each item.
    // Consumed work/allocation is never refunded on either result.
    #[cfg(feature = "management-snapshots")]
    pub(crate) fn limited_value<T: Decode>(&mut self, id: Id, maximum: usize) -> Result<T> {
        self.with_vector_limit(maximum, |reader| reader.value(id))
    }
    #[cfg(feature = "management-snapshots")]
    pub(crate) fn with_vector_limit<T>(
        &mut self,
        maximum: usize,
        read: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let previous = self.next_vector_limit;
        self.next_vector_limit = Some(previous.map_or(maximum, |old| old.min(maximum)));
        let result = read(self);
        self.next_vector_limit = previous;
        result
    }
    #[cfg(any(
        feature = "cloud-engine-marketplace",
        feature = "registry-placement",
        feature = "management-snapshots",
        feature = "caffeine",
        feature = "sns-neurons"
    ))]
    pub(crate) fn variant(&mut self, id: Id) -> Result<(u32, Id)> {
        let Node::Variant(fields) = node_at(id, self.table).map_err(|_| Error::Type)? else {
            return Err(Error::Type);
        };
        let index = self.input.count()?;
        fields.get(index).copied().ok_or(Error::Malformed)
    }
    pub(crate) fn value<T: Decode>(&mut self, id: Id) -> Result<T> {
        self.input.charge(1)?;
        if id >= 0 {
            if self.depth == 16 {
                return Err(Error::Limit);
            }
            self.depth += 1;
        }
        let result = T::read(self, id);
        if id >= 0 {
            self.depth -= 1;
        }
        result
    }

    pub(crate) fn record(
        &mut self,
        id: Id,
        mut field: impl FnMut(u32, Id, &mut Self) -> Result<bool>,
    ) -> Result<()> {
        let Node::Record(fields) = node_at(id, self.table).map_err(|_| Error::Type)? else {
            return Err(Error::Type);
        };
        for &(label, ty) in fields {
            self.input.charge(1)?;
            if !field(label, ty, self)? {
                self.skip(ty)?;
            }
        }
        Ok(())
    }

    pub(crate) fn skip(&mut self, id: Id) -> Result<()> {
        let previous = self.input.skipping;
        self.input.skipping = true;
        let result = self.discard(id);
        self.input.skipping = previous;
        result
    }

    fn discard(&mut self, id: Id) -> Result<()> {
        self.input.charge(1)?;
        match id {
            -1 | -16 => Ok(()),
            -2 => self.input.boolean().map(|_| ()),
            -3 | -4 => {
                while self.input.byte()? & 128 != 0 {}
                Ok(())
            }
            -5 | -9 => self.input.take(1).map(|_| ()),
            -6 | -10 => self.input.take(2).map(|_| ()),
            -7 | -11 | -13 => self.input.take(4).map(|_| ()),
            -8 | -12 | -14 => self.input.take(8).map(|_| ()),
            -15 => self.input.text().map(|_| ()),
            -17 => Err(Error::Malformed),
            -24 => self.input.principal().map(|_| ()),
            _ => {
                if self.depth == 16 {
                    return Err(Error::Limit);
                }
                self.depth += 1;
                let result = self.discard_node(id);
                self.depth -= 1;
                result
            }
        }
    }

    fn discard_node(&mut self, id: Id) -> Result<()> {
        match node_at(id, self.table)? {
            Node::Option(inner) => {
                if self.input.boolean()? {
                    self.discard(*inner)?;
                }
            }
            Node::Vector(inner) => {
                let count = self.input.count()?;
                if count > self.input.limits.vector {
                    return Err(Error::Limit);
                }
                for _ in 0..count {
                    self.discard(*inner)?;
                }
            }
            Node::Record(fields) => {
                for (_, inner) in fields {
                    self.discard(*inner)?;
                }
            }
            Node::Variant(fields) => {
                let index = self.input.count()?;
                let (_, inner) = fields.get(index).ok_or(Error::Malformed)?;
                self.discard(*inner)?;
            }
            Node::Service(_) => {
                self.input.principal()?;
            }
            Node::Function { .. } => {
                if self.input.byte()? != 1 {
                    return Err(Error::Unsupported);
                }
                self.input.principal()?;
                self.input.text()?;
            }
        }
        Ok(())
    }
}

// Validate the generated static declaration at compilation, not on each message.
// Incoming type-table validation remains a separate, metered trust boundary.
pub(crate) const fn schema_valid(schema: &Schema) -> bool {
    match schema {
        Schema::Primitive(id) => (*id >= -17 && *id <= -1) || *id == -24,
        Schema::Vector(inner) | Schema::StrictOption(inner) => schema_valid(inner),
        Schema::Record(fields) => fields_valid(fields),
        #[cfg(any(
            feature = "cloud-engine-marketplace",
            feature = "registry-placement",
            feature = "management-snapshots",
            feature = "caffeine",
            feature = "sns-neurons"
        ))]
        Schema::Variant(fields) => fields_valid(fields),
    }
}

const fn fields_valid(fields: &[(u32, &Schema)]) -> bool {
    let mut index = 0;
    while index < fields.len() {
        if (index > 0 && fields[index - 1].0 >= fields[index].0) || !schema_valid(fields[index].1) {
            return false;
        }
        index += 1;
    }
    true
}

pub(super) fn decode<T: Decode>(bytes: &[u8], limits: Limits) -> Result<T> {
    if bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    const { assert!(schema_valid(&T::SCHEMA), "invalid static projection schema") };
    let mut input = Input {
        bytes,
        offset: 0,
        limits,
        header: true,
        skipping: false,
    };
    let (table, root) = header(&mut input, false)?;
    let mut reader = Reader {
        table: &table,
        input,
        depth: 0,
        #[cfg(feature = "management-snapshots")]
        next_vector_limit: None,
    };
    let value = reader.value::<T>(root)?;
    if reader.input.offset != bytes.len() {
        return Err(Error::Trailing);
    }
    Ok(value)
}

impl Decode for String {
    const SCHEMA: Schema = Schema::Primitive(-15);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -15 {
            return Err(Error::Type);
        }
        let text = reader.input.text()?;
        reader.input.storage(text.len())?;
        let mut result = String::new();
        result
            .try_reserve_exact(text.len())
            .map_err(|_| Error::Allocation)?;
        result.push_str(text);
        Ok(result)
    }
}
#[cfg(any(
    feature = "cloud-engine-marketplace",
    feature = "management-snapshots",
    feature = "caffeine",
    feature = "sns-neurons"
))]
impl Decode for () {
    const SCHEMA: Schema = Schema::Primitive(-1);
    fn read(_: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id == -1 { Ok(()) } else { Err(Error::Type) }
    }
}

#[cfg(any(
    feature = "cloud-engine-marketplace",
    feature = "caffeine",
    feature = "management"
))]
impl Decode for super::Nat {
    const SCHEMA: Schema = Schema::Primitive(-3);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -3 {
            return Err(Error::Type);
        }
        let start = reader.input.offset;
        while reader.input.byte()? & 128 != 0 {}
        let count = reader.input.offset - start;
        // Meter the conversion pass and both logical scratch/magnitude storage
        // before either allocation. Round magnitude up to 64-bit words so this
        // allowance is conservative on both native and Wasm digit widths.
        reader.input.charge(count)?;
        let magnitude = count
            .checked_mul(7)
            .and_then(|bits| bits.checked_add(63))
            .map(|bits| bits / 64)
            .and_then(|words| words.checked_mul(8))
            .ok_or(Error::Limit)?;
        reader.input.storage(magnitude)?;
        let mut digits = reader.input.reserve::<u8>(count)?;
        digits.extend(
            reader.input.bytes[start..reader.input.offset]
                .iter()
                .map(|byte| byte & 127),
        );
        let value = num_bigint::BigUint::from_radix_le(&digits, 128).ok_or(Error::Malformed)?;
        Ok(Self(value))
    }
}
impl Decode for Principal {
    const SCHEMA: Schema = Schema::Primitive(-24);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -24 && !matches!(node_at(id, reader.table), Ok(Node::Service(_))) {
            return Err(Error::Type);
        }
        reader.input.principal()
    }
}
impl Decode for bool {
    const SCHEMA: Schema = Schema::Primitive(-2);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -2 {
            return Err(Error::Type);
        }
        reader.input.boolean()
    }
}
#[cfg(any(
    feature = "management-chunks",
    feature = "management-snapshots",
    feature = "caffeine",
    feature = "sns-neurons"
))]
impl Decode for u8 {
    const SCHEMA: Schema = Schema::Primitive(-5);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -5 {
            return Err(Error::Type);
        }
        reader.input.byte()
    }
}
macro_rules! fixed {
    ($ty:ty, $id:literal) => {
        impl Decode for $ty {
            const SCHEMA: Schema = Schema::Primitive($id);
            fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
                if id != $id {
                    return Err(Error::Type);
                }
                Ok(Self::from_le_bytes(
                    reader
                        .input
                        .take(size_of::<Self>())?
                        .try_into()
                        .map_err(|_| Error::Malformed)?,
                ))
            }
        }
    };
}
fixed!(u64, -8);
fixed!(i64, -12);
fixed!(f64, -14);
#[cfg(any(feature = "management-snapshots", feature = "sns-neurons"))]
fixed!(i32, -11);
#[cfg(feature = "management-snapshots")]
fixed!(f32, -13);

#[cfg(feature = "management-snapshots")]
impl Decode for u128 {
    const SCHEMA: Schema = Schema::Primitive(-3);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -3 {
            return Err(Error::Type);
        }
        let mut value = 0u128;
        let mut shift = 0u32;
        loop {
            let byte = reader.input.byte()?;
            let payload = u128::from(byte & 127);
            if shift >= 128 {
                if payload != 0 {
                    return Err(Error::Limit);
                }
            } else {
                if payload > (u128::MAX >> shift) {
                    return Err(Error::Limit);
                }
                value |= payload << shift;
            }
            if byte < 128 {
                return Ok(value);
            }
            shift = shift.saturating_add(7);
        }
    }
}

#[cfg(feature = "management-snapshots")]
pub(crate) trait KnownOptional: Sized {
    fn read_known(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self>;
}
#[cfg(feature = "management-snapshots")]
impl<T: Decode> KnownOptional for Option<T> {
    fn read_known(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if matches!(id, -1 | -16) {
            reader.input.charge(1)?;
            return Ok(None);
        }
        if matches!(node_at(id, reader.table), Ok(Node::Option(_))) {
            reader.value(id)
        } else {
            reader.value(id).map(Some)
        }
    }
}

#[cfg(feature = "management-snapshots")]
struct ByteVector(Vec<u8>);
#[cfg(feature = "management-snapshots")]
impl Decode for ByteVector {
    const SCHEMA: Schema = <Vec<u8> as Decode>::SCHEMA;
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        // Check even empty vectors: no element read would otherwise establish
        // that this is the known byte shape rather than an incompatible vector.
        if !matches!(node_at(id, reader.table), Ok(Node::Vector(-5))) {
            return Err(Error::Type);
        }
        Vec::<u8>::read(reader, id).map(Self)
    }
}
#[cfg(feature = "management-snapshots")]
impl Reader<'_, '_> {
    pub(crate) fn known_optional_bytes(&mut self, id: Id) -> Result<Option<Vec<u8>>> {
        <Option<ByteVector> as KnownOptional>::read_known(self, id)
            .map(|value| value.map(|bytes| bytes.0))
    }
}
impl<T: Decode> Decode for Vec<T> {
    const SCHEMA: Schema = Schema::Vector(&T::SCHEMA);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        reader.vector_with(id, |reader, inner| reader.value(inner))
    }
}
impl Reader<'_, '_> {
    fn vector_with<T>(
        &mut self,
        id: Id,
        read: impl Fn(&mut Self, Id) -> Result<T>,
    ) -> Result<Vec<T>> {
        let Node::Vector(inner) = node_at(id, self.table).map_err(|_| Error::Type)? else {
            return Err(Error::Type);
        };
        let inner = *inner;
        let count = self.input.count()?;
        let maximum = self.input.limits.vector;
        #[cfg(feature = "management-snapshots")]
        let maximum = self
            .next_vector_limit
            .take()
            .map_or(maximum, |field| field.min(maximum));
        if count > maximum {
            return Err(Error::Limit);
        }
        // Admit traversal before reserving attacker-selected vector storage.
        self.input.charge(count)?;
        let mut result = self.input.reserve(count)?;
        for _ in 0..count {
            result.push(read(self, inner)?);
        }
        Ok(result)
    }
}
#[cfg(feature = "management-snapshots")]
pub(crate) trait KnownElements: Sized {
    fn read_known(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self>;
}
#[cfg(feature = "management-snapshots")]
impl<T: Decode> KnownElements for Vec<Option<T>> {
    fn read_known(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        reader.vector_with(id, |reader, inner| {
            <Option<T> as KnownOptional>::read_known(reader, inner)
        })
    }
}
impl<T: Decode> Decode for Option<T> {
    const SCHEMA: Schema = Schema::StrictOption(&T::SCHEMA);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        let Node::Option(inner) = node_at(id, reader.table).map_err(|_| Error::Type)? else {
            return Err(Error::Type);
        };
        let inner = *inner;
        if reader.input.boolean()? {
            reader.value(inner).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(any(feature = "management-snapshots", feature = "caffeine"))]
pub(crate) fn decode_empty(bytes: &[u8], limits: Limits) -> Result<()> {
    if bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    let mut input = Input {
        bytes,
        offset: 0,
        limits,
        header: true,
        skipping: false,
    };
    header(&mut input, true)?;
    if input.offset != bytes.len() {
        return Err(Error::Trailing);
    }
    Ok(())
}

#[cfg(feature = "caffeine")]
impl Decode for crate::caffeine::Int {
    const SCHEMA: Schema = Schema::Primitive(-4);
    fn read(reader: &mut Reader<'_, '_>, id: Id) -> Result<Self> {
        if id != -4 {
            return Err(Error::Type);
        }
        let start = reader.input.offset;
        let last = loop {
            let byte = reader.input.byte()?;
            if byte & 128 == 0 {
                break byte;
            }
        };
        let count = reader.input.offset - start;
        let bits = count.checked_mul(7).ok_or(Error::Limit)?;
        let magnitude = bits
            .checked_add(64)
            .and_then(|n| (n / 64).checked_mul(8))
            .ok_or(Error::Limit)?;
        // Scratch digits, unsigned magnitude, sign boundary and subtraction
        // output are all admitted before invoking upstream magnitude conversion.
        reader
            .input
            .charge(count.checked_mul(4).ok_or(Error::Limit)?)?;
        reader
            .input
            .storage(magnitude.checked_mul(4).ok_or(Error::Limit)?)?;
        let mut digits = reader.input.reserve::<u8>(count)?;
        digits.extend(
            reader.input.bytes[start..reader.input.offset]
                .iter()
                .map(|byte| byte & 127),
        );
        let unsigned = num_bigint::BigUint::from_radix_le(&digits, 128).ok_or(Error::Malformed)?;
        let mut value = num_bigint::BigInt::from(unsigned);
        if last & 64 != 0 {
            value -= num_bigint::BigInt::from(1_u8) << bits;
        }
        Ok(crate::caffeine::Int(value))
    }
}
