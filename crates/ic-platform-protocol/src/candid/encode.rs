//! Private, two-pass writer for explicitly selected declarations. The first
//! pass admits the complete output before reserving any output storage.

use super::wire::{Schema, schema_valid};
use candidate_types::Principal;

/// Complete output admission failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Complete output exceeded its inclusive limit or usize capacity.
    OutputLimit,
    /// Allocation of already admitted output failed.
    Allocation,
    /// The selected static declaration exceeds the private table capacity.
    Schema,
}
impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "snapshot encoding refusal: {self:?}")
    }
}
impl std::error::Error for EncodeError {}
type Result<T> = std::result::Result<T, EncodeError>;

pub(crate) trait Encode {
    const SCHEMA: Schema;
    fn write(&self, writer: &mut Writer<'_>) -> Result<()>;
}
pub(crate) struct Writer<'a> {
    output: Option<&'a mut Vec<u8>>,
    count: usize,
    maximum: usize,
}
impl Writer<'_> {
    pub(crate) fn bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.count = self
            .count
            .checked_add(bytes.len())
            .filter(|n| *n <= self.maximum)
            .ok_or(EncodeError::OutputLimit)?;
        if let Some(output) = &mut self.output {
            output.extend_from_slice(bytes);
        }
        Ok(())
    }

    pub(crate) fn unsigned(&mut self, mut value: u64) -> Result<()> {
        loop {
            let byte = (value & 127) as u8;
            value >>= 7;
            self.bytes(&[byte | if value != 0 { 128 } else { 0 }])?;
            if value == 0 {
                return Ok(());
            }
        }
    }

    fn signed(&mut self, mut value: i64) -> Result<()> {
        loop {
            let byte = (value & 127) as u8;
            value >>= 7;
            let done = (value == 0 && byte & 64 == 0) || (value == -1 && byte & 64 != 0);
            self.bytes(&[byte | if done { 0 } else { 128 }])?;
            if done {
                return Ok(());
            }
        }
    }
}
struct Table<'a> {
    nodes: [Option<&'a Schema>; 128],
    used: usize,
}
impl<'a> Table<'a> {
    fn reference(&mut self, schema: &'a Schema) -> Result<i64> {
        if let Schema::Primitive(id) = schema {
            return Ok(*id);
        }
        if let Some(index) = self.nodes[..self.used]
            .iter()
            .position(|node| *node == Some(schema))
        {
            return Ok(index as i64);
        }
        if self.used == self.nodes.len() {
            return Err(EncodeError::Schema);
        }
        let index = self.used;
        self.nodes[index] = Some(schema);
        self.used += 1;
        match schema {
            Schema::Vector(inner) | Schema::StrictOption(inner) => {
                self.reference(inner)?;
            }
            Schema::Record(fields) | Schema::Variant(fields) => {
                for (_, inner) in *fields {
                    self.reference(inner)?;
                }
            }
            Schema::Primitive(_) => unreachable!(),
        }
        Ok(index as i64)
    }

    fn header(&mut self, root: i64, writer: &mut Writer<'_>) -> Result<()> {
        writer.bytes(b"DIDL")?;
        writer.unsigned(self.used as u64)?;
        for index in 0..self.used {
            match self.nodes[index].ok_or(EncodeError::Schema)? {
                Schema::Vector(inner) | Schema::StrictOption(inner) => {
                    writer.signed(if matches!(self.nodes[index], Some(Schema::Vector(_))) {
                        -19
                    } else {
                        -18
                    })?;
                    writer.signed(self.reference(inner)?)?;
                }
                Schema::Record(fields) | Schema::Variant(fields) => {
                    writer.signed(if matches!(self.nodes[index], Some(Schema::Record(_))) {
                        -20
                    } else {
                        -21
                    })?;
                    writer.unsigned(fields.len() as u64)?;
                    for (label, inner) in *fields {
                        writer.unsigned(u64::from(*label))?;
                        writer.signed(self.reference(inner)?)?;
                    }
                }
                Schema::Primitive(_) => return Err(EncodeError::Schema),
            }
        }
        writer.unsigned(1)?;
        writer.signed(root)
    }
}

pub(crate) fn encode<T: Encode>(value: &T, maximum: usize) -> Result<Vec<u8>> {
    const {
        assert!(schema_valid(&T::SCHEMA));
    }
    let mut table = Table {
        nodes: [None; 128],
        used: 0,
    };
    let root = table.reference(&T::SCHEMA)?;
    let mut counter = Writer {
        output: None,
        count: 0,
        maximum,
    };
    table.header(root, &mut counter)?;
    value.write(&mut counter)?;
    let size = counter.count;
    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| EncodeError::Allocation)?;
    let mut writer = Writer {
        output: Some(&mut output),
        count: 0,
        maximum: size,
    };
    table.header(root, &mut writer)?;
    value.write(&mut writer)?;
    debug_assert_eq!(writer.count, size);
    Ok(output)
}
impl Encode for Principal {
    const SCHEMA: Schema = Schema::Primitive(-24);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.bytes(&[1, self.as_slice().len() as u8])?;
        writer.bytes(self.as_slice())
    }
}
impl Encode for u64 {
    const SCHEMA: Schema = Schema::Primitive(-8);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.bytes(&self.to_le_bytes())
    }
}
impl Encode for u8 {
    const SCHEMA: Schema = Schema::Primitive(-5);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.bytes(&[*self])
    }
}
impl Encode for u128 {
    const SCHEMA: Schema = Schema::Primitive(-3);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        let mut value = *self;
        loop {
            let byte = (value & 127) as u8;
            value >>= 7;
            writer.bytes(&[byte | if value != 0 { 128 } else { 0 }])?;
            if value == 0 {
                return Ok(());
            }
        }
    }
}
macro_rules! integer {
    ($ty:ty, $id:literal) => {
        impl Encode for $ty {
            const SCHEMA: Schema = Schema::Primitive($id);
            fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
                writer.bytes(&self.to_le_bytes())
            }
        }
    };
}

integer!(i32, -11);

#[cfg(feature = "sns-neurons")]
integer!(u32, -7);

integer!(i64, -12);

impl Encode for bool {
    const SCHEMA: Schema = Schema::Primitive(-2);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.bytes(&[u8::from(*self)])
    }
}
impl<T: Encode> Encode for Option<T> {
    const SCHEMA: Schema = Schema::StrictOption(&T::SCHEMA);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.bytes(&[u8::from(self.is_some())])?;
        if let Some(value) = self {
            value.write(writer)?;
        }
        Ok(())
    }
}
impl<T: Encode> Encode for Vec<T> {
    const SCHEMA: Schema = Schema::Vector(&T::SCHEMA);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.unsigned(self.len() as u64)?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl Encode for String {
    const SCHEMA: Schema = Schema::Primitive(-15);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.unsigned(self.len() as u64)?;
        writer.bytes(self.as_bytes())
    }
}
#[cfg(any(feature = "caffeine", feature = "management"))]
impl Encode for super::Nat {
    const SCHEMA: Schema = Schema::Primitive(-3);
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        let groups = self.0.bits().div_ceil(7).max(1);
        for group in 0..groups {
            let mut byte = 0;
            for bit in 0..7 {
                byte |= u8::from(self.0.bit(group * 7 + bit)) << bit;
            }
            writer.bytes(&[byte | if group + 1 < groups { 128 } else { 0 }])?;
        }
        Ok(())
    }
}

impl<T: Encode> Encode for &T {
    const SCHEMA: Schema = T::SCHEMA;
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        (*self).write(writer)
    }
}

#[cfg(feature = "ledger")]
impl Encode for [u8; 32] {
    const SCHEMA: Schema = Schema::Vector(&Schema::Primitive(-5));
    fn write(&self, writer: &mut Writer<'_>) -> Result<()> {
        writer.unsigned(32)?;
        writer.bytes(self)
    }
}
