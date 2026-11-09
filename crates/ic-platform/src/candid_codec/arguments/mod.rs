//! Argument sequences are distinct from Candid tuple-record values.
use super::decoder::{Decode, Reader};
use super::{Encode, Error, Type, Writer, header};

/// Trusted mapping for an argument sequence, distinct from a tuple-record value.
pub trait EncodeArgs {
    /// Schemas in argument order.
    const TYPES: &'static [&'static Type];
    /// Write each argument's payload in schema order.
    fn write_args(&self, output: &mut Writer) -> Result<(), Error>;
}
/// Trusted typed argument reader. Implementations are supplied for tuples up to eight.
pub trait DecodeArgs<'a>: Sized {
    /// Number of expected arguments.
    const COUNT: usize;
    /// Read selected arguments, applying typed missing-argument rules.
    fn read_args(reader: &mut Reader<'a>, types: &[i64]) -> Result<Self, super::decoder::Error>;
}
impl EncodeArgs for () {
    const TYPES: &'static [&'static Type] = &[];
    fn write_args(&self, _: &mut Writer) -> Result<(), Error> {
        Ok(())
    }
}
impl<'a> DecodeArgs<'a> for () {
    const COUNT: usize = 0;
    fn read_args(_: &mut Reader<'a>, _: &[i64]) -> Result<Self, super::decoder::Error> {
        Ok(())
    }
}
macro_rules! arguments {
    ($($t:ident:$index:tt),+) => {
        impl<$($t: Encode),+> EncodeArgs for ($($t,)+) {
            const TYPES: &'static [&'static Type] = &[$($t::TYPE),+];
            fn write_args(&self, output: &mut Writer) -> Result<(), Error> {
                $(self.$index.write(output)?;)+
                Ok(())
            }
        }
        impl<'a, $($t: Decode<'a>),+> DecodeArgs<'a> for ($($t,)+) {
            const COUNT: usize = <Self as EncodeArgs>::TYPES.len();
            fn read_args(reader: &mut Reader<'a>, types: &[i64]) -> Result<Self, super::decoder::Error> {
                Ok(($(match types.get($index) {
                    Some(id) => reader.value::<$t>(*id)?,
                    None => <$t as Decode<'a>>::missing()?,
                },)+))
            }
        }
    };
}
arguments!(A:0);
arguments!(A:0, B:1);
arguments!(A:0, B:1, C:2);
arguments!(A:0, B:1, C:2, D:3);
arguments!(A:0, B:1, C:2, D:3, E:4);
arguments!(A:0, B:1, C:2, D:3, E:4, F:5);
arguments!(A:0, B:1, C:2, D:3, E:4, F:5, G:6);
arguments!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7);

/// Encode zero or more arguments within `limit` bytes.
pub fn encode_args<T: EncodeArgs>(values: &T, limit: usize) -> Result<Vec<u8>, Error> {
    let mut output = Writer {
        bytes: Vec::new(),
        limit,
    };
    header(T::TYPES, &mut output)?;
    values.write_args(&mut output)?;
    Ok(output.bytes)
}

#[cfg(test)]
mod tests;
