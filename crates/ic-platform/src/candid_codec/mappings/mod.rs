//! Single declarations and mappings for Rust structs and payload variants.
//! Labels must be supplied in increasing Candid field-id order. This codec
//! uses explicit mappings instead of deriving wire codecs from Serde.

/// Implement the owned codec for an existing named-field struct.
/// Supply fields in increasing numeric Candid-label order, including every field.
///
/// Alternatively, declare a concrete Rust struct and its mapping together:
/// ```
/// ic_platform::candid_record! {
///     /// A required external protocol record, without a Serde dependency.
///     #[derive(Debug, PartialEq)]
///     pub struct Reply {
///         /// Explicit Candid field identifier.
///         97 => pub amount: u64,
///         98 => pub(crate) memo: Option<String>,
///     }
/// }
/// ```
/// Type and field attributes apply to the Rust declaration; numeric labels,
/// not Serde renames or Rust names, determine the Candid schema. Put conditional
/// attributes that remove a type or field on the whole macro invocation instead,
/// so Rust and wire declarations remain together. Generic declarations are not
/// supported; concrete field types may use generic containers or recursive types.
/// Declaration labels are integer literals; their ordering is checked at compile time:
/// ```compile_fail
/// ic_platform::candid_record! { struct Invalid { 2 => a: u8, 1 => b: u8 } }
/// ```
/// ```compile_fail
/// ic_platform::candid_record! { struct Duplicate { 1 => a: u8, 1 => b: u8 } }
/// ```
#[macro_export]
macro_rules! candid_record {
    ($(#[$attribute:meta])* $visibility:vis struct $name:ident {
        $($(#[$field_attribute:meta])* $label:literal => $field_visibility:vis $field:ident : $field_type:ty),* $(,)?
    }) => {
        $(#[$attribute])*
        $visibility struct $name {
            $($(#[$field_attribute])* $field_visibility $field: $field_type),*
        }
        $crate::candid_record!(@labels $($label),*);
        $crate::candid_record!($name { $($label => $field: $field_type),* });
    };
    (@labels $($label:expr),* $(,)?) => {
        const _: () = {
            let labels: &[u32] = &[$($label),*];
            let mut index = 1;
            while index < labels.len() {
                assert!(labels[index - 1] < labels[index], "Candid labels must be strictly increasing and unique");
                index += 1;
            }
        };
    };
    ($ty:ty { $($label:expr => $field:ident : $field_type:ty),* $(,)? }) => {
        impl $crate::candid_codec::Encode for $ty {
            const TYPE: &'static $crate::candid_codec::Type = &$crate::candid_codec::Type::Deferred(|| { static SCHEMA: $crate::candid_codec::Type = $crate::candid_codec::Type::Record(&[
                $(($label, <$field_type as $crate::candid_codec::Encode>::TYPE)),*
            ]); &SCHEMA });
            fn write(&self, output: &mut $crate::candid_codec::Writer) -> ::core::result::Result<(), $crate::candid_codec::Error> {
                $($crate::candid_codec::Encode::write(&self.$field, output)?;)*
                ::core::result::Result::Ok(())
            }
        }
        impl<'a> $crate::candid_codec::decoder::Decode<'a> for $ty {
            fn read(reader: &mut $crate::candid_codec::decoder::Reader<'a>, wire: i64) -> ::core::result::Result<Self, $crate::candid_codec::decoder::Error> {
                $(let mut $field: ::core::option::Option<$field_type> = ::core::option::Option::None;)*
                reader.record(wire, false, |label, inner, reader| {
                    $(if label == $label { $field = ::core::option::Option::Some(reader.value::<$field_type>(inner)?); return ::core::result::Result::Ok(true); })*
                    ::core::result::Result::Ok(false)
                })?;
                ::core::result::Result::Ok(Self { $($field: match $field { ::core::option::Option::Some(value) => value, ::core::option::Option::None => <$field_type as $crate::candid_codec::decoder::Decode<'a>>::missing()? }),* })
            }
        }
    };
}

/// Implement the owned codec for an enum with unit, single-payload or named-field variants.
/// Supply alternatives and named fields in increasing numeric Candid-label order.
///
/// The declaration form emits both the Rust enum and the existing codec mapping:
/// ```
/// ic_platform::candid_variant! {
///     #[derive(Debug, PartialEq)]
///     pub enum Reply {
///         /// A unit alternative.
///         65 => Empty,
///         66 => Text(String),
///         67 => Details { 97 => count: u64, 98 => memo: Option<String> },
///     }
/// }
/// ```
/// A single payload can itself be a tuple, such as `Pair((u64, String))`.
/// Declaration attributes follow [`candid_record!`]'s contract. Labels at both
/// the variant and named-field levels must be strictly increasing and unique:
/// ```compile_fail
/// ic_platform::candid_variant! { enum Invalid { 1 => A, 1 => B } }
/// ```
/// ```compile_fail
/// ic_platform::candid_variant! { enum Invalid { 1 => A { 2 => a: u8, 1 => b: u8 } } }
/// ```
#[macro_export]
macro_rules! candid_variant {
    ($(#[$attribute:meta])* $visibility:vis enum $name:ident {
        $($(#[$variant_attribute:meta])* $label:literal => $variant:ident $(($payload:ty))?
            $({ $($(#[$field_attribute:meta])* $field_label:literal => $field:ident : $field_type:ty),* $(,)? })?),+ $(,)?
    }) => {
        $(#[$attribute])*
        $visibility enum $name {
            $($(#[$variant_attribute])* $variant $(($payload))?
                $({ $($(#[$field_attribute])* $field: $field_type),* })?),+
        }
        $crate::candid_record!(@labels $($label),+);
        $($($crate::candid_record!(@labels $($field_label),*);)?)+
        $crate::candid_variant!($name {
            $($label => $variant $(($payload))? $({ $($field_label => $field: $field_type),* })?),+
        });
    };
    ($ty:ty { $($label:expr => $variant:ident $(($payload:ty))? $({ $($field_label:expr => $field:ident : $field_type:ty),* $(,)? })?),+ $(,)? }) => {
        impl $crate::candid_codec::Encode for $ty {
            const TYPE: &'static $crate::candid_codec::Type = &$crate::candid_codec::Type::Deferred(|| { static SCHEMA: $crate::candid_codec::Type = $crate::candid_codec::Type::Variant(&[
                $(($label, $crate::candid_variant!(@schema $(($payload))? $({ $($field_label => $field : $field_type),* })?))),+
            ]); &SCHEMA });
            fn write(&self, output: &mut $crate::candid_codec::Writer) -> ::core::result::Result<(), $crate::candid_codec::Error> {
                $( $crate::candid_variant!(@write self, output, $label, $variant $(($payload))? $({ $($field_label => $field : $field_type),* })?); )+
                ::core::result::Result::Err($crate::candid_codec::Error::Schema)
            }
        }
        impl<'a> $crate::candid_codec::decoder::Decode<'a> for $ty {
            fn read(reader: &mut $crate::candid_codec::decoder::Reader<'a>, wire: i64) -> ::core::result::Result<Self, $crate::candid_codec::decoder::Error> {
                reader.variant(wire, |label, inner, reader| {
                    $(if label == $label { return $crate::candid_variant!(@read reader, inner, $variant $(($payload))? $({ $($field_label => $field : $field_type),* })?); })+
                    ::core::result::Result::Err($crate::candid_codec::decoder::Error::Type)
                })
            }
        }
    };
    (@schema ($payload:ty)) => { <$payload as $crate::candid_codec::Encode>::TYPE };
    (@schema { $($label:expr => $field:ident : $ty:ty),* }) => {
        &$crate::candid_codec::Type::Record(&[$(($label, <$ty as $crate::candid_codec::Encode>::TYPE)),*])
    };
    (@schema) => { <() as $crate::candid_codec::Encode>::TYPE };
    (@tag $out:ident, $label:expr) => {
        let $crate::candid_codec::Type::Variant(fields) = <Self as $crate::candid_codec::Encode>::TYPE.resolved()? else {
            return ::core::result::Result::Err($crate::candid_codec::Error::Schema);
        };
        let index = fields.binary_search_by_key(&$label, |field| field.0).map_err(|_| $crate::candid_codec::Error::Schema)?;
        $out.unsigned(index as u128)?;
    };
    (@write $value:ident, $out:ident, $label:expr, $variant:ident ($payload:ty)) => {
        if let Self::$variant(value) = $value {
            $crate::candid_variant!(@tag $out, $label);
            return $crate::candid_codec::Encode::write(value, $out);
        }
    };
    (@write $value:ident, $out:ident, $label:expr, $variant:ident { $($field_label:expr => $field:ident : $ty:ty),* }) => {
        if let Self::$variant { $($field),* } = $value {
            $crate::candid_variant!(@tag $out, $label);
            $($crate::candid_codec::Encode::write($field, $out)?;)*
            return ::core::result::Result::Ok(());
        }
    };
    (@write $value:ident, $out:ident, $label:expr, $variant:ident) => {
        if let Self::$variant = $value {
            $crate::candid_variant!(@tag $out, $label);
            return ::core::result::Result::Ok(());
        }
    };
    (@read $reader:ident, $inner:ident, $variant:ident ($payload:ty)) => {
        ::core::result::Result::Ok(Self::$variant($reader.value::<$payload>($inner)?))
    };
    (@read $reader:ident, $inner:ident, $variant:ident { $($label:expr => $field:ident : $ty:ty),* }) => {{
        $(let mut $field: ::core::option::Option<$ty> = ::core::option::Option::None;)*
        $reader.record($inner, false, |label, inner, reader| {
            $(if label == $label { $field = ::core::option::Option::Some(reader.value::<$ty>(inner)?); return ::core::result::Result::Ok(true); })*
            ::core::result::Result::Ok(false)
        })?;
        ::core::result::Result::Ok(Self::$variant { $($field: match $field {
            ::core::option::Option::Some(value) => value,
            ::core::option::Option::None => <$ty as $crate::candid_codec::decoder::Decode<'_>>::missing()?,
        }),* })
    }};
    (@read $reader:ident, $inner:ident, $variant:ident) => {{
        $reader.value::<()>($inner)?;
        ::core::result::Result::Ok(Self::$variant)
    }};
}

#[cfg(test)]
mod tests;
