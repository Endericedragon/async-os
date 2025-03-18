use core::arch::asm;

mod ms_client;

fn main() {
    greeting_through_syscall();
}

fn greeting_through_syscall() {
    let message = "Greetings!\n".as_bytes();
    let len = message.len();
    const SYS_WRITE: usize = 64;
    let rua = 1usize;
    // const STDOUT: usize = 1;
    unsafe {
        asm!(
            "ecall",
            in("a7") SYS_WRITE,
            in("a0") rua,
            in("a1") message.as_ptr(),
            in("a2") len,
        );
    }
}
