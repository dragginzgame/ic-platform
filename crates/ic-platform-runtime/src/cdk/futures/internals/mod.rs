//! Entrypoint context adapters backed by Runtime's sole executor.

pub use crate::executor::{
    in_tracking_executor_context as in_executor_context,
    in_tracking_query_executor_context as in_query_executor_context,
};
