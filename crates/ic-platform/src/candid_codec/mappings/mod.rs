//! Declarative mappings for existing Rust structs and payload variants.
//! Labels must be supplied in increasing Candid field-id order. This codec
//! uses explicit mappings instead of deriving wire codecs from Serde.

/// Implement the owned codec for an existing named-field struct.
/// Supply fields in increasing numeric Candid-label order, including every field.
#[macro_export]
macro_rules! candid_record {
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
#[macro_export]
macro_rules! candid_variant {
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
