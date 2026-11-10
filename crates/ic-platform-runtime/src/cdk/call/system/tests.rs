//! Small native seam for observing the new transport's supplier calls.

use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::cdk::call) enum Event {
    Cost(u64, u64),
    Balance,
    Awaited {
        callee: Vec<u8>,
        method: String,
        reply_env: usize,
        reject_env: usize,
        shared_callback: bool,
    },
    Cleanup(usize),
    Oneway {
        callee: Vec<u8>,
        method: String,
    },
    Append(Vec<u8>),
    Cycles(u128),
    Timeout(u32),
    Perform,
}

pub(in crate::cdk::call) struct Mock {
    pub cost: u128,
    pub balance: u128,
    pub status: u32,
    pub events: Vec<Event>,
    pub environments: Vec<usize>,
    pub reply_code: u32,
    pub reply_data: Vec<u8>,
    pub reject_message: String,
    pub message_reads: usize,
    constructing: Option<usize>,
}

impl Default for Mock {
    fn default() -> Self {
        Self {
            cost: 10,
            balance: u128::MAX,
            status: 0,
            events: Vec::new(),
            environments: Vec::new(),
            reply_code: 0,
            reply_data: Vec::new(),
            reject_message: String::new(),
            message_reads: 0,
            constructing: None,
        }
    }
}

thread_local! {
    static MOCK: RefCell<Mock> = RefCell::new(Mock::default());
}

pub(in crate::cdk::call) fn with_mock<R>(f: impl FnOnce(&mut Mock) -> R) -> R {
    MOCK.with(|mock| f(&mut mock.borrow_mut()))
}

pub(in crate::cdk::call) fn cost_call(method: u64, payload: u64) -> u128 {
    with_mock(|mock| {
        mock.events.push(Event::Cost(method, payload));
        mock.cost
    })
}

pub(in crate::cdk::call) fn canister_liquid_cycle_balance() -> u128 {
    with_mock(|mock| {
        mock.events.push(Event::Balance);
        mock.balance
    })
}

#[expect(
    unsafe_code,
    reason = "The test seam retains the production callback-registration signature without dereferencing pointers"
)]
pub(in crate::cdk::call) unsafe fn call_new(
    callee: &[u8],
    method: &str,
    reply: unsafe extern "C" fn(usize),
    reply_env: usize,
    reject: unsafe extern "C" fn(usize),
    reject_env: usize,
) {
    with_mock(|mock| {
        assert!(mock.constructing.is_none());
        mock.constructing = Some(reply_env);
        mock.events.push(Event::Awaited {
            callee: callee.to_vec(),
            method: method.to_owned(),
            reply_env,
            reject_env,
            shared_callback: std::ptr::fn_addr_eq(reply, reject),
        });
    });
}

#[expect(
    unsafe_code,
    reason = "The test seam retains the production cleanup-registration signature without invoking callbacks"
)]
pub(in crate::cdk::call) unsafe fn call_on_cleanup(
    _: unsafe extern "C" fn(usize),
    environment: usize,
) {
    with_mock(|mock| {
        assert_eq!(mock.constructing, Some(environment));
        mock.events.push(Event::Cleanup(environment));
    });
}

pub(in crate::cdk::call) fn call_new_oneway(callee: &[u8], method: &str) {
    with_mock(|mock| {
        assert!(mock.constructing.is_none());
        mock.events.push(Event::Oneway {
            callee: callee.to_vec(),
            method: method.to_owned(),
        });
    });
}

pub(in crate::cdk::call) fn call_data_append(bytes: &[u8]) {
    with_mock(|mock| mock.events.push(Event::Append(bytes.to_vec())));
}

pub(in crate::cdk::call) fn call_cycles_add128(cycles: u128) {
    with_mock(|mock| mock.events.push(Event::Cycles(cycles)));
}

pub(in crate::cdk::call) fn call_with_best_effort_response(timeout: u32) {
    with_mock(|mock| mock.events.push(Event::Timeout(timeout)));
}

pub(in crate::cdk::call) fn call_perform() -> u32 {
    with_mock(|mock| {
        mock.events.push(Event::Perform);
        if let Some(environment) = mock.constructing.take()
            && mock.status == 0
        {
            mock.environments.push(environment);
        }
        mock.status
    })
}

pub(in crate::cdk::call) fn msg_reject_code() -> u32 {
    with_mock(|mock| {
        mock.message_reads += 1;
        mock.reply_code
    })
}

pub(in crate::cdk::call) fn msg_arg_data() -> Vec<u8> {
    with_mock(|mock| {
        mock.message_reads += 1;
        mock.reply_data.clone()
    })
}

pub(in crate::cdk::call) fn msg_reject_msg() -> String {
    with_mock(|mock| {
        mock.message_reads += 1;
        mock.reject_message.clone()
    })
}

pub(in crate::cdk::call) fn trap(message: impl AsRef<str>) -> ! {
    panic!("{}", message.as_ref())
}
