//! Merge repeated declarations without unfolding recursive schema graphs.
use super::{Error, Type};
use std::ptr;
pub(super) type Aliases = Vec<(&'static Type, &'static Type)>;
pub(super) fn compact(table: &mut Vec<&'static Type>) -> Result<Aliases, Error> {
    // Registration already validated these declarations. With at most one
    // entry there are no distinct declarations to merge, including self-cycles.
    if table.len() <= 1 {
        return Ok(table.iter().map(|ty| (*ty, *ty)).collect());
    }
    let mut classes: Vec<usize> = (0..table.len()).collect();
    loop {
        let id = |ty: &'static Type| -> Result<i64, Error> {
            let ty = ty.resolved()?;
            match ty {
                Type::Primitive(code) => Ok(i64::from(*code)),
                _ => table
                    .iter()
                    .position(|entry| ptr::eq(*entry, ty))
                    .map(|i| classes[i] as i64)
                    .ok_or(Error::Schema),
            }
        };
        let mut signatures = Vec::with_capacity(table.len());
        for (index, ty) in table.iter().enumerate() {
            let mut key = Vec::new();
            match ty {
                Type::Opt(inner) | Type::Vec(inner) => {
                    key.push(if matches!(ty, Type::Opt(_)) { -18 } else { -19 });
                    key.push(id(inner)?);
                }
                Type::Record(fields) | Type::Variant(fields) => {
                    key.push(if matches!(ty, Type::Record(_)) {
                        -20
                    } else {
                        -21
                    });
                    for (label, ty) in *fields {
                        key.push(i64::from(*label));
                        key.push(id(ty)?);
                    }
                }
                Type::Function {
                    args,
                    results,
                    annotations,
                } => {
                    key.extend([-22, args.len() as i64]);
                    for ty in *args {
                        key.push(id(ty)?);
                    }
                    key.push(results.len() as i64);
                    for ty in *results {
                        key.push(id(ty)?);
                    }
                    key.extend(annotations.iter().map(|byte| i64::from(*byte)));
                }
                Type::Service(methods) => {
                    key.push(-23);
                    for (name, ty) in *methods {
                        key.push(name.len() as i64);
                        key.extend(name.bytes().map(i64::from));
                        key.push(id(ty)?);
                    }
                }
                _ => return Err(Error::Schema),
            }
            signatures.push((key, index));
        }
        signatures.sort_unstable();
        let mut next = classes.clone();
        let mut previous = None;
        let mut representative = 0;
        for (key, index) in &signatures {
            if previous != Some(key) {
                representative = *index;
                previous = Some(key);
            }
            next[*index] = representative;
        }
        if next == classes {
            break;
        }
        classes = next;
    }
    let aliases: Aliases = table
        .iter()
        .enumerate()
        .map(|(i, ty)| (*ty, table[classes[i]]))
        .collect();
    let mut index = 0;
    table.retain(|_| {
        let keep = classes[index] == index;
        index += 1;
        keep
    });
    Ok(aliases)
}
