//! One fixture selects either the existing raw transport or the owned request.
#[cfg(feature = "ic-call")]
use ic_call::{Call, WaitPolicy};
use ic_platform::Principal;
#[cfg(not(feature = "ic-call"))]
use ic_platform::cdk::call::Call;

#[cfg(feature = "ic-call")]
pub fn bounded_wait(target: Principal, method: &str) -> Call<'static, 'static> {
    // Returning an owned dynamic method exercises Canic's reported gap.
    Call::new(
        target,
        method.to_owned(),
        WaitPolicy::Bounded {
            timeout_seconds: 300,
        },
    )
}
#[cfg(not(feature = "ic-call"))]
pub fn bounded_wait(target: Principal, method: &str) -> Call<'_, 'static> {
    Call::bounded_wait(target, method)
}

#[cfg(feature = "ic-call")]
pub fn unbounded_wait(target: Principal, method: &str) -> Call<'static, 'static> {
    Call::new(target, method.to_owned(), WaitPolicy::Unbounded)
}
#[cfg(not(feature = "ic-call"))]
pub fn unbounded_wait(target: Principal, method: &str) -> Call<'_, 'static> {
    Call::unbounded_wait(target, method)
}

// Test-only spelling adapter; the shipped raw API stays unchanged.
#[cfg(not(feature = "ic-call"))]
pub trait Execute: std::future::IntoFuture {
    fn execute(self) -> Self::IntoFuture
    where
        Self: Sized,
    {
        self.into_future()
    }
}
#[cfg(not(feature = "ic-call"))]
impl<'m, 'a> Execute for Call<'m, 'a> {}
