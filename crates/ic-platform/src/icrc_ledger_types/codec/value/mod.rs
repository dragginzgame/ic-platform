use super::super::value::*;
use crate::candid_codec::{Int, Nat};
use serde_bytes::ByteBuf;
crate::candid_variant!(ICRC3Value {3654863 => Int(Int), 3850876 => Map(ICRC3Map), 3900609 => Nat(Nat), 737307005 => Blob(ByteBuf), 936573133 => Text(String), 3099385209 => Array(Vec<ICRC3Value>)});
crate::candid_variant!(Value {3654863 => Int(Int), 3850876 => Map(Map), 3900609 => Nat(Nat), 699868735 => Nat64(u64), 737307005 => Blob(ByteBuf), 936573133 => Text(String), 3099385209 => Array(Vec<Value>)});
