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
}
macro_rules! projection {
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
    ($(#[$meta:meta])* enum $name:ident { $($(#[$variant_meta:meta])* $label:literal => $variant:ident $(($ty:ty))?),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub enum $name { $($(#[$variant_meta])* $variant $(($ty))?,)+ }
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Variant(&[$(($label, &projection!(@schema $($ty)?))),+]);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                let (label, wire) = reader.variant(id)?;
                match label {
                    $($label => projection!(@read reader, wire, $name, $variant $(, $ty)?),)+
                    _ => Err(Error::Type),
                }
            }
        }
    };
    ($(#[$meta:meta])* $name:ident { $( $(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty [$policy:ident] ),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name { $( $(#[$field_meta])* pub $field: $ty, )+ }
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Record(&[$(($label, &<$ty as Decode>::SCHEMA)),+]);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                $(let mut $field = None;)+
                reader.record(id, |label, wire, reader| {
                    match label {
                        $($label => { $field = Some(reader.value::<$ty>(wire)?); },)+
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
