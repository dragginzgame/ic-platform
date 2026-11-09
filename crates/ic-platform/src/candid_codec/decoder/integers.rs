use super::{Decode, Error, Id, Reader, Result};
use num_bigint::{BigInt, BigUint, Sign};

impl Reader<'_> {
    fn magnitude(&mut self, signed: bool) -> Result<(BigUint, bool)> {
        let start = self.pos;
        let last = loop {
            let byte = self.byte()?;
            if byte & 128 == 0 {
                break byte;
            }
        };
        let negative = signed && last & 64 != 0;
        let encoded = &self.input[start..self.pos];
        // Charge temporary radix digits and an upper bound for the bigint's
        // limb storage before asking the maintained supplier to allocate it.
        self.allocate(encoded.len())?;
        let limbs = encoded
            .len()
            .checked_mul(7)
            .ok_or(Error::Limit)?
            .div_ceil(32);
        self.allocate(
            limbs
                .checked_add(1)
                .and_then(|n| n.checked_mul(4))
                .ok_or(Error::Limit)?,
        )?;
        let digits: Vec<_> = encoded
            .iter()
            .map(|byte| (byte & 127) ^ if negative { 127 } else { 0 })
            .collect();
        let magnitude = BigUint::from_radix_le(&digits, 128).ok_or(Error::Malformed)?;
        Ok((magnitude, negative))
    }
}
impl<'a> Decode<'a> for BigUint {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        if id != -3 {
            return Err(Error::Type);
        }
        Ok(r.magnitude(false)?.0)
    }
}
impl<'a> Decode<'a> for BigInt {
    fn read(r: &mut Reader<'a>, id: Id) -> Result<Self> {
        if !matches!(id, -3 | -4) {
            return Err(Error::Type);
        }
        let (mut magnitude, negative) = r.magnitude(id == -4)?;
        if negative {
            magnitude += 1u8;
        }
        Ok(BigInt::from_biguint(
            if negative { Sign::Minus } else { Sign::Plus },
            magnitude,
        ))
    }
}
