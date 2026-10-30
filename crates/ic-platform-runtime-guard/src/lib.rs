//! Internal Cargo packaging guard for IC Platform's sole executor.
//!
//! This package contains no executor or runtime API. The facade selects its exact
//! coordinated version only when the executor feature is enabled. Its inherited
//! `links` identity refuses other executor suppliers during dependency resolution.
#![no_std]
