;; Independent peer: accept exactly five cycles, either commit or trap.
(module
  (import "ic0" "msg_cycles_accept128" (func $accept (param i64 i64 i32)))
  (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
  (import "ic0" "msg_reply" (func $reply))
  (memory (export "memory") 1)
  (func (export "canister_update accept")
    (call $accept (i64.const 0) (i64.const 5) (i32.const 0))
    (call $append (i32.const 0) (i32.const 16))
    (call $reply))
  (func (export "canister_update accept_trap")
    (call $accept (i64.const 0) (i64.const 5) (i32.const 0))
    unreachable))
