//! Bounded selected request encoding; no service pagination or limit policy.

use crate::{candid::Nat, cloud_engine::EncodeError};
use ic_platform_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// Raw caller-selected arguments for one available-node page query.
    arguments ListAvailableNodesPageArgs {
        /// Optional arbitrary natural, without enforcing service page-size policy.
        1_963_007_803 => limit: Option<Nat>,
        /// Optional canonical continuation identity.
        2_215_343_202 => start: Option<Principal>,
        /// Optional opaque selector, preserving empty text and UTF-8 bytes.
        3_623_907_301 => rejoining: Option<String>,
    }
}

/// Encode exactly one page-argument record within a complete output-byte limit.
///
/// None differs from explicit zero, an empty Principal or empty text. The whole
/// frame is measured before output allocation. Natural-number bits are borrowed
/// from upstream storage; encoding creates no intermediate magnitude buffer.
/// The caller owns service limits, rejoining policy and cursor traversal.
///
/// # Errors
/// Returns an output-limit refusal before allocation or an allocation refusal
/// when the admitted output reservation fails.
pub fn encode_list_available_nodes_page_args(
    args: &ListAvailableNodesPageArgs,
    max_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    let mut writer = ArgumentWriter {
        output: None,
        size: 0,
        max_bytes,
    };
    args.write(&mut writer)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(writer.size)
        .map_err(|_| EncodeError::Allocation)?;
    writer.output = Some(output);
    writer.size = 0;
    args.write(&mut writer)?;
    writer.output.ok_or(EncodeError::Allocation)
}

struct ArgumentWriter {
    output: Option<Vec<u8>>,
    size: usize,
    max_bytes: usize,
}

impl ArgumentWriter {
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.advance(bytes.len())?;
        if let Some(output) = &mut self.output {
            output.extend_from_slice(bytes);
        }
        Ok(())
    }
    fn advance(&mut self, count: usize) -> Result<(), EncodeError> {
        self.size = self
            .size
            .checked_add(count)
            .filter(|size| *size <= self.max_bytes)
            .ok_or(EncodeError::OutputLimit {
                max_bytes: self.max_bytes,
            })?;
        Ok(())
    }
    fn byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.bytes(&[byte])
    }
    fn uleb(&mut self, mut value: usize) -> Result<(), EncodeError> {
        loop {
            let byte = (value & 127) as u8;
            value >>= 7;
            self.byte(if value == 0 { byte } else { byte | 128 })?;
            if value == 0 {
                return Ok(());
            }
        }
    }
}

trait Argument {
    const PRIMITIVE: u8;
    fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError>;
}

impl<T: Argument> Argument for Option<T> {
    const PRIMITIVE: u8 = T::PRIMITIVE;
    fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError> {
        writer.byte(u8::from(self.is_some()))?;
        if let Some(value) = self {
            value.write(writer)?;
        }
        Ok(())
    }
}
impl Argument for String {
    const PRIMITIVE: u8 = 0x71;
    fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError> {
        writer.uleb(self.len())?;
        writer.bytes(self.as_bytes())
    }
}
impl Argument for Principal {
    const PRIMITIVE: u8 = 0x68;
    fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError> {
        writer.byte(1)?;
        writer.uleb(self.as_slice().len())?;
        writer.bytes(self.as_slice())
    }
}
impl Argument for Nat {
    const PRIMITIVE: u8 = 0x7d;
    fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError> {
        let bits = self.0.bits();
        let count =
            usize::try_from(bits.div_ceil(7).max(1)).map_err(|_| EncodeError::OutputLimit {
                max_bytes: writer.max_bytes,
            })?;
        // The measuring pass does not traverse a caller's arbitrary magnitude.
        if writer.output.is_none() {
            return writer.advance(count);
        }
        for index in 0..count {
            let offset = (index as u64) * 7;
            let mut byte = 0u8;
            for bit in 0..bits.saturating_sub(offset).min(7) {
                byte |= u8::from(self.0.bit(offset + bit)) << bit;
            }
            writer.byte(byte | if index + 1 == count { 0 } else { 128 })?;
        }
        Ok(())
    }
}
