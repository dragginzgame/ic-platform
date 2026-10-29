//! Iterative wire-nesting admission before the upstream decoder sees the header.
//!
//! This walk retains no decoded application values, performs no subtyping and
//! never rewrites bytes. It bounds all type-table paths (including unused entries)
//! and every encoded value (including skipped arguments/fields). Recursive type
//! declarations remain allowed; actual recursive values consume the depth budget.
//! Future constructors and opaque references are refused rather than passed on
//! without inspection. Upstream Candid still owns semantic/type validation.

use super::DecodeLimits;
use std::{error::Error, fmt};

/// Explicit admission allowances, separate from Candid's decoding/skip quotas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NestingLimits {
    /// Maximum constructed-type path and constructed-value depth. Scalars use
    /// depth zero; an empty container still uses one level. No default is chosen.
    pub depth: usize,
    /// Maximum admission work: consumed bytes, graph edges and visited values.
    /// This is not the upstream decoder's work unit or a remaining decode quota.
    pub work: usize,
}

/// A compact, typed refusal from admission, without disclosing input bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NestingError {
    /// Original message exceeds its existing byte allowance.
    Bytes {
        /// Observed message length.
        actual: usize,
        /// Caller-selected byte allowance.
        maximum: usize,
    },
    /// Header exceeds its existing byte allowance.
    Header {
        /// Caller-selected header byte allowance.
        maximum: usize,
    },
    /// Type table exceeds its existing entry allowance.
    Types {
        /// Declared table entries.
        actual: usize,
        /// Caller-selected entry allowance.
        maximum: usize,
    },
    /// A path through the type table exceeds the caller's nesting allowance.
    TypeDepth {
        /// Caller-selected nesting allowance.
        maximum: usize,
    },
    /// An encoded value exceeds the caller's nesting allowance.
    ValueDepth {
        /// Caller-selected nesting allowance.
        maximum: usize,
    },
    /// Admission work is exhausted, including zero-byte value expansion.
    Work {
        /// Caller-selected admission work allowance.
        maximum: usize,
    },
    /// Framing is malformed or truncated at this byte offset.
    Malformed {
        /// Offset observed before refusing framing.
        offset: usize,
    },
    /// A future constructor cannot be inspected by this admission implementation.
    Unsupported {
        /// Uninspected constructor opcode.
        opcode: i64,
    },
}
impl fmt::Display for NestingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bytes { actual, maximum } => {
                write!(f, "Candid admission bytes {actual} exceed {maximum}")
            }
            Self::Header { maximum } => {
                write!(f, "Candid admission header exceeds {maximum} bytes")
            }
            Self::Types { actual, maximum } => {
                write!(f, "Candid admission type entries {actual} exceed {maximum}")
            }
            Self::TypeDepth { maximum } => write!(f, "Candid type nesting exceeds {maximum}"),
            Self::ValueDepth { maximum } => write!(f, "Candid value nesting exceeds {maximum}"),
            Self::Work { maximum } => write!(f, "Candid nesting admission work exceeds {maximum}"),
            Self::Malformed { offset } => {
                write!(f, "invalid Candid admission framing at byte {offset}")
            }
            Self::Unsupported { opcode } => {
                write!(f, "unsupported Candid admission constructor {opcode}")
            }
        }
    }
}
impl Error for NestingError {}

type Result<T> = std::result::Result<T, NestingError>;

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
    end: usize,
    header: bool,
    remaining: usize,
    limits: NestingLimits,
}
impl Cursor<'_> {
    fn malformed(&self) -> NestingError {
        NestingError::Malformed { offset: self.pos }
    }
    fn charge(&mut self, amount: usize) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or(NestingError::Work {
                maximum: self.limits.work,
            })?;
        Ok(())
    }
    fn take(&mut self, count: usize) -> Result<&[u8]> {
        let end = self
            .pos
            .checked_add(count)
            .ok_or_else(|| self.malformed())?;
        if end > self.end {
            return Err(if self.header && self.end < self.bytes.len() {
                NestingError::Header { maximum: self.end }
            } else {
                self.malformed()
            });
        }
        self.charge(count)?;
        let bytes = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn leb(&mut self, signed: bool) -> Result<u64> {
        let mut value = 0u64;
        for i in 0..10 {
            let byte = self.byte()?;
            let payload = byte & 0x7f;
            if i == 9
                && (byte & 0x80 != 0
                    || if signed {
                        payload != 0 && payload != 0x7f
                    } else {
                        payload > 1
                    })
            {
                return Err(self.malformed());
            }
            value |= u64::from(payload) << (i * 7);
            if byte & 0x80 == 0 {
                let bits = (i + 1) * 7;
                if signed && bits < 64 && byte & 0x40 != 0 {
                    value |= u64::MAX << bits;
                }
                return Ok(value);
            }
        }
        Err(self.malformed())
    }
    fn count(&mut self) -> Result<usize> {
        usize::try_from(self.leb(false)?).map_err(|_| self.malformed())
    }
    fn index(&mut self) -> Result<i64> {
        let value = self.leb(true)? as i64;
        if value < -17 && value != -24 {
            return Err(self.malformed());
        }
        Ok(value)
    }
    fn text(&mut self) -> Result<()> {
        let len = self.count()?;
        self.take(len)?;
        Ok(())
    }
    fn principal(&mut self) -> Result<()> {
        if self.byte()? != 1 {
            return Err(self.malformed());
        }
        let len = self.count()?;
        if len > 29 {
            return Err(self.malformed());
        }
        self.take(len)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Opt,
    Vec,
    Record,
    Variant,
    Func,
    Service,
}
struct Node {
    kind: Kind,
    children: Vec<i64>,
}

fn header(cursor: &mut Cursor<'_>, limits: DecodeLimits) -> Result<(Vec<Node>, Vec<i64>)> {
    if cursor.take(4)? != b"DIDL" {
        return Err(cursor.malformed());
    }
    let count = cursor.count()?;
    if count > limits.type_entries {
        return Err(NestingError::Types {
            actual: count,
            maximum: limits.type_entries,
        });
    }
    let mut types = Vec::new();
    for _ in 0..count {
        let opcode = cursor.leb(true)? as i64;
        let kind = match opcode {
            -18 => Kind::Opt,
            -19 => Kind::Vec,
            -20 => Kind::Record,
            -21 => Kind::Variant,
            -22 => Kind::Func,
            -23 => Kind::Service,
            value if value < -24 => return Err(NestingError::Unsupported { opcode: value }),
            _ => return Err(cursor.malformed()),
        };
        let mut children = Vec::new();
        match kind {
            Kind::Opt | Kind::Vec => children.push(cursor.index()?),
            Kind::Record | Kind::Variant => {
                let fields = cursor.count()?;
                let mut previous = None;
                for _ in 0..fields {
                    let id = u32::try_from(cursor.leb(false)?).map_err(|_| cursor.malformed())?;
                    if previous.is_some_and(|prev| id <= prev) {
                        return Err(cursor.malformed());
                    }
                    previous = Some(id);
                    children.push(cursor.index()?);
                }
            }
            Kind::Func => {
                for _ in 0..2 {
                    let fields = cursor.count()?;
                    for _ in 0..fields {
                        children.push(cursor.index()?);
                    }
                }
                match cursor.byte()? {
                    0 => (),
                    1 => {
                        if !(1..=3).contains(&cursor.byte()?) {
                            return Err(cursor.malformed());
                        }
                    }
                    _ => return Err(cursor.malformed()),
                }
            }
            Kind::Service => {
                let methods = cursor.count()?;
                for _ in 0..methods {
                    cursor.text()?;
                    children.push(cursor.index()?);
                }
            }
        }
        types.push(Node { kind, children });
    }
    let count = cursor.count()?;
    let mut args = Vec::new();
    for _ in 0..count {
        args.push(cursor.index()?);
    }
    for index in args
        .iter()
        .chain(types.iter().flat_map(|node| &node.children))
    {
        if *index >= 0 && usize::try_from(*index).map_or(true, |i| i >= types.len()) {
            return Err(cursor.malformed());
        }
    }
    Ok((types, args))
}

fn type_depth(cursor: &mut Cursor<'_>, types: &[Node]) -> Result<()> {
    let mut active = vec![false; types.len()];
    let mut stack = Vec::new();
    for root in 0..types.len() {
        cursor.charge(1)?;
        if cursor.limits.depth == 0 {
            return Err(NestingError::TypeDepth { maximum: 0 });
        }
        active[root] = true;
        stack.push((root, 0usize));
        while let Some((index, next)) = stack.last_mut() {
            let node = &types[*index];
            if *next == node.children.len() {
                active[*index] = false;
                stack.pop();
                continue;
            }
            let child = node.children[*next];
            *next += 1;
            cursor.charge(1)?;
            if child < 0 {
                continue;
            }
            let child = usize::try_from(child).map_err(|_| cursor.malformed())?;
            if active[child] {
                continue;
            }
            if stack.len() >= cursor.limits.depth {
                return Err(NestingError::TypeDepth {
                    maximum: cursor.limits.depth,
                });
            }
            active[child] = true;
            stack.push((child, 0));
        }
    }
    Ok(())
}

fn width(ty: i64) -> Option<usize> {
    match ty {
        -1 | -16 => Some(0),
        -5 | -9 => Some(1),
        -6 | -10 => Some(2),
        -7 | -11 | -13 => Some(4),
        -8 | -12 | -14 => Some(8),
        _ => None,
    }
}

enum Task<'a> {
    Values(&'a [i64], usize, usize),
    Value(i64, usize),
    Repeat(i64, usize, usize),
}
fn values(cursor: &mut Cursor<'_>, types: &[Node], args: &[i64]) -> Result<()> {
    let mut stack = vec![Task::Values(args, 0, 0)];
    while let Some(task) = stack.pop() {
        let (ty, depth) = match task {
            Task::Values(items, next, depth) => {
                if next == items.len() {
                    continue;
                }
                stack.push(Task::Values(items, next + 1, depth));
                (items[next], depth)
            }
            Task::Repeat(ty, count, depth) => {
                if count == 0 {
                    continue;
                }
                stack.push(Task::Repeat(ty, count - 1, depth));
                (ty, depth)
            }
            Task::Value(ty, depth) => (ty, depth),
        };
        cursor.charge(1)?;
        if ty < 0 {
            if let Some(width) = width(ty) {
                cursor.take(width)?;
            } else {
                match ty {
                    -2 => {
                        if cursor.byte()? > 1 {
                            return Err(cursor.malformed());
                        }
                    }
                    -3 | -4 => while cursor.byte()? & 0x80 != 0 {},
                    -15 => cursor.text()?,
                    -24 => cursor.principal()?,
                    _ => return Err(cursor.malformed()),
                }
            }
            continue;
        }
        if depth >= cursor.limits.depth {
            return Err(NestingError::ValueDepth {
                maximum: cursor.limits.depth,
            });
        }
        let depth = depth + 1;
        let node = &types[usize::try_from(ty).map_err(|_| cursor.malformed())?];
        match node.kind {
            Kind::Opt => match cursor.byte()? {
                0 => (),
                1 => stack.push(Task::Value(node.children[0], depth)),
                _ => return Err(cursor.malformed()),
            },
            Kind::Vec => {
                let count = cursor.count()?;
                let ty = node.children[0];
                if let Some(width) = width(ty) {
                    cursor.charge(count)?;
                    let size = count.checked_mul(width).ok_or_else(|| cursor.malformed())?;
                    cursor.take(size)?;
                } else {
                    stack.push(Task::Repeat(ty, count, depth));
                }
            }
            Kind::Record => stack.push(Task::Values(&node.children, 0, depth)),
            Kind::Variant => {
                let index = cursor.count()?;
                let child = *node.children.get(index).ok_or_else(|| cursor.malformed())?;
                stack.push(Task::Value(child, depth));
            }
            Kind::Func => {
                if cursor.byte()? != 1 {
                    return Err(cursor.malformed());
                }
                cursor.principal()?;
                cursor.text()?;
            }
            Kind::Service => cursor.principal()?,
        }
    }
    if cursor.pos != cursor.bytes.len() {
        return Err(cursor.malformed());
    }
    Ok(())
}

/// Admit the complete wire envelope using bounded heap stacks, before constructing
/// upstream `IDLDeserialize`. Work is charged independently of decoder quotas.
///
/// This is wire-nesting admission, not a replacement decoder or a guarantee about
/// arbitrary custom Rust visitors/expected schemas. Low-level Candid APIs remain
/// caller-owned. No future/opaque wire form is silently passed without inspection.
///
/// # Errors
/// Returns typed byte/header/type/depth/work or unsupported/framing refusals.
pub fn check(bytes: &[u8], wire: DecodeLimits, nesting: NestingLimits) -> Result<()> {
    if bytes.len() > wire.reply_bytes {
        return Err(NestingError::Bytes {
            actual: bytes.len(),
            maximum: wire.reply_bytes,
        });
    }
    let mut cursor = Cursor {
        bytes,
        pos: 0,
        end: bytes.len().min(wire.header_bytes),
        header: true,
        remaining: nesting.work,
        limits: nesting,
    };
    let (types, args) = header(&mut cursor, wire)?;
    type_depth(&mut cursor, &types)?;
    cursor.header = false;
    cursor.end = bytes.len();
    values(&mut cursor, &types, &args)
}

#[cfg(test)]
mod tests;
