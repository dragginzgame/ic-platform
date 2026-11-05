#[test]
fn interrupted_pc_is_read_from_the_signal_context() {
    let mut context: libc::ucontext_t = unsafe { std::mem::zeroed() };
    context.uc_mcontext.gregs[libc::REG_RIP as usize] = 0x1234_5678;
    assert_eq!(
        unsafe { super::signal_instruction_pointer((&context as *const libc::ucontext_t).cast()) },
        Some(0x1234_5678)
    );
    assert_eq!(
        unsafe { super::signal_instruction_pointer(std::ptr::null()) },
        None
    );
}
