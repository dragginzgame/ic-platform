//! Native external-service export from owned mappings. No parser or registration.
//! Consumers emit descriptors from the same catalogue as their mounted endpoints.
//! Compare the result structurally with a frozen DID using `didc check --strict`.
use super::{Error, Type, Writer, arguments::EncodeArgs, register};
use std::ptr;

/// External Candid method mode; caller visibility does not select a codec.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    /// Replicated update.
    Update,
    /// Query.
    Query,
    /// No reply; the result sequence must be empty.
    Oneway,
    /// Composite query.
    CompositeQuery,
}
impl Mode {
    fn annotations(self) -> &'static [u8] {
        match self {
            Self::Update => &[],
            Self::Query => &[1],
            Self::Oneway => &[2],
            Self::CompositeQuery => &[3],
        }
    }
}

/// One explicitly declared external method, using argument/result sequences.
#[derive(Clone, Copy, Debug)]
pub struct Method {
    name: &'static str,
    args: &'static [&'static Type],
    results: &'static [&'static Type],
    mode: Mode,
}
impl Method {
    /// Build from the actual mounted argument/result types. `()` means no values;
    /// `(T,)` means one value, even when `T` is a tuple record or unit.
    pub const fn new<A: EncodeArgs, R: EncodeArgs>(name: &'static str, mode: Mode) -> Self {
        Self {
            name,
            args: A::TYPES,
            results: R::TYPES,
            mode,
        }
    }
}

/// Generate deterministic DID text, bounded by `maximum_bytes`. Names/aliases and
/// whitespace are presentation only: use structural equality, not text equality.
/// Duplicate methods and invalid owned schemas refuse. This describes explicit
/// descriptors; it cannot discover exports or prove a consumer mounted them.
pub fn export(methods: &[Method], maximum_bytes: usize) -> Result<String, Error> {
    if methods.len() > maximum_bytes {
        return Err(Error::Limit);
    }
    let mut methods: Vec<_> = methods.iter().collect();
    methods.sort_by_key(|method| method.name);
    if methods.windows(2).any(|pair| pair[0].name == pair[1].name) {
        return Err(Error::Schema);
    }
    let mut table = Vec::new();
    for method in &methods {
        if matches!(method.mode, Mode::Oneway) && !method.results.is_empty() {
            return Err(Error::Schema);
        }
        for ty in method.args.iter().chain(method.results) {
            register(ty, &mut table, maximum_bytes)?;
        }
    }
    let mut out = Writer {
        bytes: Vec::new(),
        limit: maximum_bytes,
    };
    for (index, ty) in table.iter().enumerate() {
        out.bytes(format!("type T{index} = ").as_bytes())?;
        definition(ty, &table, &mut out)?;
        out.bytes(b";\n")?;
    }
    out.bytes(b"service : {\n")?;
    for method in methods {
        quoted(method.name, &mut out)?;
        out.bytes(b" : ")?;
        function(
            method.args,
            method.results,
            method.mode.annotations(),
            &table,
            &mut out,
        )?;
        out.bytes(b";\n")?;
    }
    out.bytes(b"}\n")?;
    String::from_utf8(out.bytes).map_err(|_| Error::Schema)
}
fn quoted(name: &str, out: &mut Writer) -> Result<(), Error> {
    out.bytes(b"\"")?;
    for byte in name.bytes() {
        if (32..127).contains(&byte) && !matches!(byte, b'"' | b'\\') {
            out.bytes(&[byte])?;
        } else {
            out.bytes(format!("\\{byte:02x}").as_bytes())?;
        }
    }
    out.bytes(b"\"")
}
fn reference(ty: &'static Type, table: &[&'static Type], out: &mut Writer) -> Result<(), Error> {
    let ty = ty.resolved()?;
    if let Type::Primitive(code) = ty {
        let name = match code {
            -1 => "null",
            -2 => "bool",
            -3 => "nat",
            -4 => "int",
            -5 => "nat8",
            -6 => "nat16",
            -7 => "nat32",
            -8 => "nat64",
            -9 => "int8",
            -10 => "int16",
            -11 => "int32",
            -12 => "int64",
            -13 => "float32",
            -14 => "float64",
            -15 => "text",
            -16 => "reserved",
            -17 => "empty",
            -24 => "principal",
            _ => return Err(Error::Schema),
        };
        out.bytes(name.as_bytes())
    } else {
        let index = table
            .iter()
            .position(|entry| ptr::eq(*entry, ty))
            .ok_or(Error::Schema)?;
        out.bytes(format!("T{index}").as_bytes())
    }
}
fn sequence(
    types: &[&'static Type],
    table: &[&'static Type],
    out: &mut Writer,
) -> Result<(), Error> {
    out.bytes(b"(")?;
    for (index, ty) in types.iter().enumerate() {
        if index != 0 {
            out.bytes(b", ")?;
        }
        reference(ty, table, out)?;
    }
    out.bytes(b")")
}
fn function(
    args: &[&'static Type],
    results: &[&'static Type],
    modes: &[u8],
    table: &[&'static Type],
    out: &mut Writer,
) -> Result<(), Error> {
    sequence(args, table, out)?;
    out.bytes(b" -> ")?;
    sequence(results, table, out)?;
    out.bytes(match modes {
        [] => b"",
        [1] => b" query",
        [2] => b" oneway",
        [3] => b" composite_query",
        _ => return Err(Error::Schema),
    })
}
fn definition(ty: &'static Type, table: &[&'static Type], out: &mut Writer) -> Result<(), Error> {
    match ty {
        Type::Opt(inner) | Type::Vec(inner) => {
            out.bytes(if matches!(ty, Type::Opt(_)) {
                b"opt "
            } else {
                b"vec "
            })?;
            reference(inner, table, out)
        }
        Type::Record(fields) | Type::Variant(fields) => {
            out.bytes(if matches!(ty, Type::Record(_)) {
                b"record {"
            } else {
                b"variant {"
            })?;
            for (id, field) in *fields {
                out.bytes(format!("{id} : ").as_bytes())?;
                reference(field, table, out)?;
                out.bytes(b";")?;
            }
            out.bytes(b"}")
        }
        Type::Function {
            args,
            results,
            annotations,
        } => {
            out.bytes(b"func ")?;
            function(args, results, annotations, table, out)
        }
        Type::Service(methods) => {
            out.bytes(b"service {")?;
            for (name, ty) in *methods {
                quoted(name, out)?;
                out.bytes(b" : ")?;
                reference(ty, table, out)?;
                out.bytes(b";")?;
            }
            out.bytes(b"}")
        }
        _ => Err(Error::Schema),
    }
}

#[cfg(test)]
mod tests;
