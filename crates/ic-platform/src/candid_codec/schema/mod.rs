//! Merge repeated declarations without unfolding recursive schema graphs.
use super::{Error, Type};
use std::ptr;
pub(super) type Aliases = Vec<(&'static Type, usize)>;
pub(super) fn compact(table: &mut Vec<&'static Type>) -> Result<Aliases, Error> {
    // Registration already validated these declarations. With at most one
    // entry there are no distinct declarations to merge, including self-cycles.
    if table.len() <= 1 {
        return Ok(table.iter().enumerate().map(|(id, ty)| (*ty, id)).collect());
    }
    let mut classes: Vec<usize> = (0..table.len()).collect();
    let mut next = vec![0; table.len()];
    // Preserve signature buffers across refinement rounds. Sorting carries each
    // declaration's original index, so stable wire-table ordering is unchanged.
    let mut signatures: Vec<(Vec<i64>, usize)> =
        (0..table.len()).map(|index| (Vec::new(), index)).collect();
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
        for (key, index) in &mut signatures {
            key.clear();
            let ty = table[*index];
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
        }
        signatures.sort_unstable();
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
        std::mem::swap(&mut classes, &mut next);
    }
    let mut compact_id = 0;
    for (index, class) in classes.iter().enumerate() {
        if index == *class {
            next[index] = compact_id;
            compact_id += 1;
        }
    }
    let aliases: Aliases = table
        .iter()
        .enumerate()
        .map(|(i, ty)| (*ty, next[classes[i]]))
        .collect();
    let mut index = 0;
    table.retain(|_| {
        let keep = classes[index] == index;
        index += 1;
        keep
    });
    Ok(aliases)
}
