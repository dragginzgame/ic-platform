//! Arbitrary-size integer arithmetic belongs to num-bigint; this module owns
//! only Candid's signed/unsigned LEB128 framing.
use super::{Encode, Error, Type, Writer};
use num_bigint::{BigInt, BigUint, Sign};

impl Encode for BigUint {
    const TYPE: &'static Type = &Type::Primitive(-3);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        let count = usize::try_from(self.bits().max(1).div_ceil(7)).map_err(|_| Error::Limit)?;
        if count > output.limit.saturating_sub(output.bytes.len()) {
            return Err(Error::Limit);
        }
        let digits = self.to_radix_le(128);
        for (index, digit) in digits.iter().enumerate() {
            output.bytes(&[digit | if index + 1 == digits.len() { 0 } else { 128 }])?;
        }
        Ok(())
    }
}
impl Encode for BigInt {
    const TYPE: &'static Type = &Type::Primitive(-4);
    fn write(&self, output: &mut Writer) -> Result<(), Error> {
        let negative = self.sign() == Sign::Minus;
        // Preflight a lower bound before materializing a radix conversion.
        let lower = usize::try_from(self.bits().saturating_sub(1).max(1).div_ceil(7))
            .map_err(|_| Error::Limit)?;
        if lower > output.limit.saturating_sub(output.bytes.len()) {
            return Err(Error::Limit);
        }
        let mut magnitude = self.magnitude().clone();
        if negative {
            magnitude -= 1u8;
        }
        let mut digits = magnitude.to_radix_le(128);
        if negative {
            for digit in &mut digits {
                *digit ^= 127;
            }
        }
        if (digits.last().copied().unwrap_or(0) & 64 != 0) != negative {
            digits.push(if negative { 127 } else { 0 });
        }
        for (index, digit) in digits.iter().enumerate() {
            output.bytes(&[digit | if index + 1 == digits.len() { 0 } else { 128 }])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
