// One declaration emits fields, schema and reader dispatch. Optional policy is
// explicit at each field. Candid-compatible fallback is deliberately not part
// of this private strict projection declaration.
macro_rules! absent {
    (required) => {
        Err(Error::Type)
    };
    (strict_optional) => {
        Ok(None)
    };
    (known_optional) => {
        Ok(None)
    };
    (known_optional_bytes) => {
        Ok(None)
    };
    (known_elements) => {
        Err(Error::Type)
    };
}
macro_rules! projection {
    ($(#[$meta:meta])* both $(($feature:literal))? $name:ident { $($(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty [$policy:ident $(, max = $maximum:expr)?]),+ $(,)? }) => {
        projection! { $(#[$meta])* $name { $($(#[$field_meta])* $label => $field: $ty [$policy $(, max = $maximum)?]),+ } }
        $(#[cfg(feature = $feature)])?
        impl Encode for $name {
            const SCHEMA: Schema = <Self as Decode>::SCHEMA;
            fn write(&self, writer: &mut Writer<'_>) -> Result<(), crate::candid::encode::EncodeError> { $(self.$field.write(writer)?;)+ Ok(()) }
        }
    };
    ($(#[$meta:meta])* both $(($feature:literal))? enum $name:ident { $($(#[$variant_meta:meta])* $label:literal => $variant:ident $(($ty:ty))?),+ $(,)? }) => {
        projection! { $(#[$meta])* enum $name { $($(#[$variant_meta])* $label => $variant $(($ty))?),+ } }
        $(#[cfg(feature = $feature)])?
        impl Encode for $name {
            const SCHEMA: Schema = <Self as Decode>::SCHEMA;
            fn write(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
                const LABELS: &[u32] = &[$($label),+];
                match self {
                    $(projection!(@pattern $name, $variant $(, $ty)?) => {
                        writer.unsigned(LABELS.iter().position(|label| *label == $label).ok_or(EncodeError::Schema)? as u64)?;
                        projection!(@write self, writer, $name, $variant $(, $ty)?)
                    }),+
                }
            }
        }
    };
    (@pattern $name:ident, $variant:ident) => { $name::$variant };
    (@pattern $name:ident, $variant:ident, $ty:ty) => { $name::$variant(_) };
    (@write $value:ident, $writer:ident, $name:ident, $variant:ident) => { Ok(()) };
    (@write $value:ident, $writer:ident, $name:ident, $variant:ident, $ty:ty) => {
        if let $name::$variant(value) = $value { value.write($writer) } else { Err(EncodeError::Schema) }
    };
    ($(#[$meta:meta])* encoding $name:ident { $($(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name { $($(#[$field_meta])* pub $field: $ty,)+ }
        impl Encode for $name {
            const SCHEMA: Schema = Schema::Record(&[$(($label, &<$ty as Encode>::SCHEMA)),+]);
            fn write(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
                $(self.$field.write(writer)?;)+
                Ok(())
            }
        }
        impl $name {
            /// Encode one complete argument. Size admission precedes output allocation.
            /// # Errors
            /// Refuses an output over the inclusive bound or allocation failure.
            pub fn encode(&self, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
                crate::candid::encode::encode(self, maximum_bytes)
            }
        }
    };
    (@value $reader:ident, $wire:ident, $ty:ty, known_optional) => { <$ty as crate::candid::KnownOptional>::read_known($reader, $wire) };
    (@value $reader:ident, $wire:ident, $ty:ty, known_optional_bytes) => { $reader.known_optional_bytes($wire) };
    (@value $reader:ident, $wire:ident, $ty:ty, known_elements) => { <$ty as crate::candid::KnownElements>::read_known($reader, $wire) };
    (@value $reader:ident, $wire:ident, $ty:ty, $policy:ident) => { $reader.value::<$ty>($wire) };
    (@value $reader:ident, $wire:ident, $ty:ty, $policy:ident, $maximum:expr) => {
        $reader.with_vector_limit($maximum, |$reader| projection!(@value $reader, $wire, $ty, $policy))
    };
    ($(#[$meta:meta])* arguments $name:ident { $($(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name { $($(#[$field_meta])* pub $field: $ty,)+ }
        impl $name {
            fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError> {
                const LABELS: &[u32] = &[$($label),+];
                const { assert!(LABELS.len() <= 63); let mut i = 1; while i < LABELS.len() {
                    assert!(LABELS[i - 1] < LABELS[i]); i += 1;
                } }
                writer.bytes(b"DIDL")?;
                writer.uleb(LABELS.len() + 1)?;
                writer.byte(0x6c)?;
                writer.uleb(LABELS.len())?;
                for (index, label) in LABELS.iter().enumerate() {
                    writer.uleb(*label as usize)?;
                    // This selected graph has three option entries; their
                    // positive type references fit one signed LEB byte.
                    writer.byte((index + 1) as u8)?;
                }
                $(writer.bytes(&[0x6e, <$ty as Argument>::PRIMITIVE])?;)+
                writer.bytes(&[1, 0])?;
                $(self.$field.write(writer)?;)+
                Ok(())
            }
        }
    };
    (@schema) => { <() as Decode>::SCHEMA };
    (@schema $ty:ty) => { <$ty as Decode>::SCHEMA };
    (@read $reader:ident, $wire:ident, $name:ident, $variant:ident) => {
        { $reader.value::<()>($wire)?; Ok($name::$variant) }
    };
    (@read $reader:ident, $wire:ident, $name:ident, $variant:ident, $ty:ty) => {
        Ok($name::$variant($reader.value::<$ty>($wire)?))
    };
    (@unknown $reader:ident, $wire:ident, $label:ident) => { Err(Error::Type) };
    (@unknown $reader:ident, $wire:ident, $label:ident, $unknown:ident) => {{
        $reader.skip($wire)?;
        Ok(Self::$unknown($label))
    }};
    ($(#[$meta:meta])* $(open $unknown:ident)? enum $name:ident { $($(#[$variant_meta:meta])* $label:literal => $variant:ident $(($ty:ty))?),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub enum $name {
            $($(#[$variant_meta])* $variant $(($ty))?,)+
            $(
                /// Unrecognized selected variant, identified by its wire label.
                $unknown(u32),
            )?
        }
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Variant(&[$(($label, &projection!(@schema $($ty)?))),+]);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                let (label, wire) = reader.variant(id)?;
                match label {
                    $($label => projection!(@read reader, wire, $name, $variant $(, $ty)?),)+
                    _ => projection!(@unknown reader, wire, label $(, $unknown)?),
                }
            }
        }
    };
    ($(#[$meta:meta])* $name:ident { $( $(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty [$policy:ident $(, max = $maximum:expr)?] ),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name { $( $(#[$field_meta])* pub $field: $ty, )+ }
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Record(&[$(($label, &<$ty as Decode>::SCHEMA)),+]);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                $(let mut $field = None;)+
                reader.record(id, |label, wire, reader| {
                    match label {
                        $($label => { $field = Some(projection!(@value reader, wire, $ty, $policy $(, $maximum)?)?); },)+
                        _ => return Ok(false),
                    }
                    Ok(true)
                })?;
                Ok(Self { $($field: match $field { Some(value) => value, None => absent!($policy)?, }),+ })
            }
        }
    };
}

pub(crate) use {absent, projection};
