use std::cell::RefCell;

thread_local! {
    static EVENTS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

pub fn record(event: &'static str) {
    EVENTS.with(|events| events.borrow_mut().push(event));
}

pub fn take() -> Vec<&'static str> {
    EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
}

pub fn decode_unit(bytes: Vec<u8>) {
    assert_eq!(bytes, [3]);
    record("decode");
}

pub fn encode_unit() -> Vec<u8> {
    record("encode");
    vec![4]
}

pub mod api {
    pub fn msg_arg_data() -> Vec<u8> {
        super::record("copy");
        vec![3]
    }

    pub fn msg_reply(bytes: Vec<u8>) {
        assert_eq!(bytes, [4]);
        super::record("reply");
    }
}

pub mod futures {
    pub mod internals {
        pub fn in_executor_context(action: impl FnOnce()) {
            super::super::record("update");
            action();
        }

        pub fn in_query_executor_context(action: impl FnOnce()) {
            super::super::record("query");
            action();
        }
    }
}
