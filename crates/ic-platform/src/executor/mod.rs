//! Platform's single maintained executor and its method-context identity.
//!
//! These explicit exports retain the imported task and trap contract. They are
//! runtime primitives, not a scheduling policy or a second executor. Consumers
//! own entrypoint/callback context placement and must qualify it on the IC.

mod runtime;

pub use runtime::{
    MethodHandle, TaskHandle, cancel_all_tasks_attached_to_current_method, cancel_task,
    extend_current_method_context, in_callback_executor_context_for, in_tracking_executor_context,
    in_tracking_query_executor_context, in_trap_recovery_context_for, is_recovering_from_trap,
    spawn_migratory, spawn_protected,
};
