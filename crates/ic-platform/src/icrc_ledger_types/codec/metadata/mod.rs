use super::super::metadata::MetadataValue;
use crate::candid_codec::{Int, Nat};
use serde_bytes::ByteBuf;
crate::candid_variant!(MetadataValue {3654863 => Int(Int), 3900609 => Nat(Nat), 737307005 => Blob(ByteBuf), 936573133 => Text(String)});
