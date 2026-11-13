//! Matched entropy consumer: raw Runtime calls, with either protocol codec.
use ic_platform_runtime::{
    Principal,
    call::{Call, CallFailed, WaitPolicy},
    cdk::{api, futures},
};
use std::cell::Cell;

thread_local! {
    static CLEANED: Cell<u8> = const { Cell::new(0) };
}

struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        CLEANED.set(CLEANED.get() + 1);
    }
}

fn arguments() -> Vec<u8> {
    #[cfg(feature = "reference-codec")]
    {
        // Test-only general encoder retained solely for the matched size baseline.
        ic_platform::candid_codec::arguments::encode_args(&(), 64).unwrap()
    }
    #[cfg(not(feature = "reference-codec"))]
    {
        ic_platform::raw_rand::encode().to_vec()
    }
}

fn random(bytes: &[u8]) -> [u8; 32] {
    #[cfg(feature = "reference-codec")]
    {
        use ic_platform::candid_codec::decoder::Limits as DecodeLimits;
        // Test-only general decoder; production callers use raw_rand::decode.
        ic_platform::candid_codec::decoder::decode_one::<Vec<u8>>(
            bytes,
            DecodeLimits {
                bytes: 256,
                work: 4_096,
                // The general decoder also charges its parsed type table and
                // argument metadata, in addition to the 32-byte result vector.
                allocation: 512,
                types: 8,
                header: 64,
            },
        )
        .unwrap()
        .try_into()
        .unwrap()
    }
    #[cfg(not(feature = "reference-codec"))]
    {
        ic_platform::raw_rand::decode(bytes).unwrap()
    }
}

/// Reply with the callback-cleanup count observed by this installed fixture.
#[expect(unsafe_code, reason = "The isolated canister exports its IC query ABI")]
#[unsafe(export_name = "canister_query cleaned")]
pub extern "C" fn cleaned() {
    api::msg_reply([CLEANED.get()]);
}

/// Exercise raw entropy, rejection, callback traps and pending-call cancellation.
///
/// # Panics
/// Panics if entropy decoding or cancellation assertions fail; mode 2 deliberately
/// traps the callback to exercise cleanup.
#[expect(
    unsafe_code,
    reason = "The isolated canister exports its IC update ABI"
)]
#[unsafe(export_name = "canister_update entropy")]
pub extern "C" fn entropy() {
    futures::internals::in_executor_context(|| {
        futures::spawn(async {
            let mode = api::msg_arg_data().first().copied().unwrap_or(0);
            if mode == 3 {
                // Cancel a pending raw_rand call, including stale-handle refusal.
                let count = std::rc::Rc::new(Cell::new(0));
                struct Dropped(std::rc::Rc<Cell<u8>>);
                impl Drop for Dropped {
                    fn drop(&mut self) {
                        self.0.set(self.0.get() + 1);
                    }
                }
                let spawn = || {
                    let dropped = Dropped(count.clone());
                    ic_platform_runtime::executor::spawn_migratory(async move {
                        let _dropped = dropped;
                        let args = arguments();
                        let _ = Call::new(
                            Principal::management_canister(),
                            ic_platform::raw_rand::METHOD,
                            WaitPolicy::Bounded {
                                timeout_seconds: 300,
                            },
                        )
                        .with_raw_args(&args)
                        .execute()
                        .await;
                    })
                };
                let first = spawn();
                ic_platform_runtime::executor::cancel_task(&first);
                let second = spawn();
                ic_platform_runtime::executor::cancel_task(&first);
                assert_eq!(count.get(), 1);
                ic_platform_runtime::executor::cancel_task(&second);
                api::msg_reply([count.get()]);
                return;
            }
            let _cleanup = (mode == 2).then(|| Cleanup);
            let target = if mode == 1 {
                api::canister_self()
            } else {
                Principal::management_canister()
            };
            let method = if mode == 1 {
                "missing_raw_rand_fixture"
            } else {
                ic_platform::raw_rand::METHOD
            };
            let args = arguments();
            let response = Call::new(
                target,
                method,
                WaitPolicy::Bounded {
                    timeout_seconds: 300,
                },
            )
            .with_raw_args(&args)
            .execute()
            .await;
            match response {
                Err(CallFailed::CallRejected(_)) if mode == 1 => api::msg_reply([1]),
                Err(error) => api::trap(error.to_string()),
                Ok(response) => {
                    let bytes = response.into_bytes();
                    let _random = random(&bytes);
                    if mode == 2 {
                        api::trap("raw_rand fixture callback trap");
                    }
                    // Retain the original producer bytes for native reference checks.
                    api::msg_reply(bytes);
                }
            }
        });
    });
}
