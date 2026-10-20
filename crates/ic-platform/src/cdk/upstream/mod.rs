//! Maintained non-cryptographic adapters imported from CDK 0.20.3.
//!
//! The sole executor remains registry ic-cdk-executor 2.0.0.

#[cfg(target_feature = "atomics")]
compile_error!("This version of the runtime does not support multithreading.");

pub mod api;
// Imported callback ABI and Arc ownership require these reviewed unsafe blocks.
#[allow(unsafe_code)]
pub mod call;
pub mod futures;
pub mod stable;
