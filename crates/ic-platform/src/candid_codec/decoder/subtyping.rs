//! Iterative, budgeted structural subtyping for typed function references.
use super::{Error, Id, Reader, Result, Wire};
use crate::candid_codec::{Type, register};

impl Reader<'_> {
    fn pair(&mut self, pending: &mut Vec<(Id, Id)>, pair: (Id, Id)) -> Result<()> {
        self.charge(1)?;
        self.allocate(size_of::<(Id, Id)>())?;
        pending.try_reserve(1).map_err(|_| Error::Limit)?;
        pending.push(pair);
        Ok(())
    }
    fn nullable(&self, id: Id) -> bool {
        matches!(id, -1 | -16) || matches!(self.node(id), Ok(Wire::Opt(_)))
    }
    pub(super) fn schema(&mut self, root: &'static Type) -> Result<Id> {
        let mut types = Vec::new();
        register(root, &mut types, self.work).map_err(|error| match error {
            crate::candid_codec::Error::Limit => Error::Limit,
            crate::candid_codec::Error::Schema => Error::Malformed,
        })?;
        self.charge(types.len().saturating_mul(types.len()))?;
        self.allocate(
            types
                .len()
                .checked_mul(size_of::<Wire>() + size_of::<&Type>())
                .ok_or(Error::Limit)?,
        )?;
        let offset = self.table.len();
        let id = |ty: &'static Type| -> Result<Id> {
            let ty = ty.resolved().map_err(|_| Error::Malformed)?;
            match ty {
                Type::Primitive(code) => Ok(i64::from(*code)),
                _ => types
                    .iter()
                    .position(|entry| std::ptr::eq(*entry, ty))
                    .and_then(|index| index.checked_add(offset))
                    .and_then(|index| i64::try_from(index).ok())
                    .ok_or(Error::Limit),
            }
        };
        self.table
            .try_reserve(types.len())
            .map_err(|_| Error::Limit)?;
        for ty in &types {
            let wire = match ty {
                Type::Opt(inner) => Wire::Opt(id(inner)?),
                Type::Vec(inner) => Wire::Vec(id(inner)?),
                Type::Record(fields) | Type::Variant(fields) => {
                    self.allocate(
                        fields
                            .len()
                            .checked_mul(size_of::<(u32, Id)>())
                            .ok_or(Error::Limit)?,
                    )?;
                    self.charge(fields.len().saturating_mul(types.len().saturating_add(1)))?;
                    let fields = fields
                        .iter()
                        .map(|(label, ty)| Ok((*label, id(ty)?)))
                        .collect::<Result<Vec<_>>>()?;
                    if matches!(ty, Type::Record(_)) {
                        Wire::Record(fields)
                    } else {
                        Wire::Variant(fields)
                    }
                }
                Type::Function {
                    args,
                    results,
                    annotations,
                } => {
                    let count = args.len().checked_add(results.len()).ok_or(Error::Limit)?;
                    self.allocate(count.checked_mul(size_of::<Id>()).ok_or(Error::Limit)?)?;
                    self.charge(count.saturating_mul(types.len().saturating_add(1)))?;
                    Wire::Function(
                        args.iter().map(|ty| id(ty)).collect::<Result<Vec<_>>>()?,
                        results
                            .iter()
                            .map(|ty| id(ty))
                            .collect::<Result<Vec<_>>>()?,
                        annotations.first().copied().unwrap_or(0),
                    )
                }
                Type::Service(methods) => {
                    self.allocate(
                        methods
                            .len()
                            .checked_mul(size_of::<(String, Id)>())
                            .ok_or(Error::Limit)?,
                    )?;
                    self.charge(methods.len())?;
                    let mut entries = Vec::new();
                    entries
                        .try_reserve_exact(methods.len())
                        .map_err(|_| Error::Limit)?;
                    for (name, ty) in *methods {
                        self.allocate(name.len())?;
                        entries.push(((*name).to_owned(), id(ty)?));
                    }
                    Wire::Service(entries)
                }
                Type::Primitive(_) | Type::Deferred(_) => unreachable!(),
            };
            self.table.push(wire);
        }
        id(root)
    }
    pub(super) fn subtype(&mut self, actual: Id, expected: Id) -> Result<()> {
        let mut pending = Vec::new();
        let mut seen = Vec::new();
        self.pair(&mut pending, (actual, expected))?;
        while let Some((a, b)) = pending.pop() {
            self.charge(seen.len().saturating_add(1))?;
            if a == b || b == -16 || a == -17 || (a == -3 && b == -4) || seen.contains(&(a, b)) {
                continue;
            }
            // Candid's special optional rule admits every source type; actual
            // optional values still use the value reader's validated fallback.
            if matches!(self.node(b), Ok(Wire::Opt(_))) {
                continue;
            }
            if b == -24 && matches!(self.node(a), Ok(Wire::Service(_))) {
                continue;
            }
            self.pair(&mut seen, (a, b))?;
            match (self.node(a)?, self.node(b)?) {
                (Wire::Vec(x), Wire::Vec(y)) => {
                    let pair = (*x, *y);
                    self.pair(&mut pending, pair)?;
                }
                (Wire::Record(_), Wire::Record(fields)) => {
                    let count = fields.len();
                    self.charge(count)?;
                    for index in 0..count {
                        let Wire::Record(fields) = self.node(b)? else {
                            unreachable!()
                        };
                        let (label, expected) = fields[index];
                        let Wire::Record(fields) = self.node(a)? else {
                            unreachable!()
                        };
                        let actual = fields
                            .binary_search_by_key(&label, |field| field.0)
                            .ok()
                            .map(|index| fields[index].1);
                        let comparisons = (usize::BITS - fields.len().leading_zeros()) as usize;
                        self.charge(comparisons)?;
                        match actual {
                            Some(actual) => self.pair(&mut pending, (actual, expected))?,
                            None if self.nullable(expected) => (),
                            None => return Err(Error::Type),
                        }
                    }
                }
                (Wire::Variant(fields), Wire::Variant(_)) => {
                    let count = fields.len();
                    self.charge(count)?;
                    for index in 0..count {
                        let Wire::Variant(fields) = self.node(a)? else {
                            unreachable!()
                        };
                        let (label, actual) = fields[index];
                        let Wire::Variant(fields) = self.node(b)? else {
                            unreachable!()
                        };
                        let expected = fields
                            .binary_search_by_key(&label, |field| field.0)
                            .ok()
                            .map(|index| fields[index].1);
                        let comparisons = (usize::BITS - fields.len().leading_zeros()) as usize;
                        self.charge(comparisons)?;
                        let expected = expected.ok_or(Error::Type)?;
                        self.pair(&mut pending, (actual, expected))?;
                    }
                }
                (Wire::Service(_), Wire::Service(methods)) => {
                    let count = methods.len();
                    self.charge(count)?;
                    for index in 0..count {
                        let Wire::Service(methods) = self.node(b)? else {
                            unreachable!()
                        };
                        let (name, expected) = &methods[index];
                        let expected = *expected;
                        let Wire::Service(methods) = self.node(a)? else {
                            unreachable!()
                        };
                        let actual = methods
                            .binary_search_by(|(method, _)| method.as_str().cmp(name.as_str()))
                            .ok()
                            .map(|index| methods[index].1);
                        let comparisons = name
                            .len()
                            .saturating_add(1)
                            .saturating_mul((usize::BITS - methods.len().leading_zeros()) as usize);
                        self.charge(comparisons)?;
                        let actual = actual.ok_or(Error::Type)?;
                        self.pair(&mut pending, (actual, expected))?;
                    }
                }
                (Wire::Function(aa, _, am), Wire::Function(_, br, bm)) => {
                    if am != bm {
                        return Err(Error::Type);
                    }
                    let (arguments, results) = (aa.len(), br.len());
                    self.charge(arguments.saturating_add(results))?;
                    // Input tuple is contravariant; result tuple is covariant.
                    for index in 0..arguments {
                        let Wire::Function(aa, _, _) = self.node(a)? else {
                            unreachable!()
                        };
                        let expected = aa[index];
                        let Wire::Function(ba, _, _) = self.node(b)? else {
                            unreachable!()
                        };
                        let actual = ba.get(index).copied();
                        match actual {
                            Some(actual) => self.pair(&mut pending, (actual, expected))?,
                            None if self.nullable(expected) => (),
                            None => return Err(Error::Type),
                        }
                    }
                    for index in 0..results {
                        let Wire::Function(_, br, _) = self.node(b)? else {
                            unreachable!()
                        };
                        let expected = br[index];
                        let Wire::Function(_, ar, _) = self.node(a)? else {
                            unreachable!()
                        };
                        let actual = ar.get(index).copied();
                        match actual {
                            Some(actual) => self.pair(&mut pending, (actual, expected))?,
                            None if self.nullable(expected) => (),
                            None => return Err(Error::Type),
                        }
                    }
                }
                _ => return Err(Error::Type),
            }
        }
        Ok(())
    }
}

impl Reader<'_> {
    pub(crate) fn function(
        &mut self,
        wire: Id,
        signature: &'static Type,
    ) -> Result<crate::candid_codec::references::Func> {
        let signature = signature.resolved().map_err(|_| Error::Malformed)?;
        if !matches!(signature, Type::Function { .. }) {
            return Err(Error::Malformed);
        }
        if !matches!(self.node(wire)?, Wire::Function(..)) {
            return Err(Error::Type);
        }
        let expected = self.schema(signature)?;
        self.subtype(wire, expected)?;
        self.enter(|reader| {
            if reader.byte()? != 1 {
                return Err(Error::Unsupported);
            }
            let principal = crate::principal::Principal::try_from_slice(reader.principal_bytes()?)
                .map_err(|_| Error::Malformed)?;
            let method = reader.text()?;
            reader.allocate(method.len())?;
            Ok(crate::candid_codec::references::Func {
                principal,
                method: method.to_owned(),
            })
        })
    }
}
