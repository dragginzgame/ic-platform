pub mod sdk {
    pub mod futures {
        pub mod internals {
            pub fn in_executor_context(task: impl FnOnce()) {
                task();
            }
            pub fn in_query_executor_context(task: impl FnOnce()) {
                task();
            }
        }
    }
}

#[ic_platform_macros::inspect_message(crate = "crate::sdk")]
fn inspect() {}
#[ic_platform_macros::init(crate = "crate::sdk")]
fn init() {}
#[ic_platform_macros::post_upgrade(crate = "crate::sdk")]
fn upgrade() {}

pub fn value() -> u8 {
    2
}
