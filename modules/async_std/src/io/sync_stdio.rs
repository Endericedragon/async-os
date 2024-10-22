use super::ax_console_write_byte;
use core::fmt::{Result, Write, Arguments};

pub struct SyncStdout;

impl Write for SyncStdout {
    fn write_str(&mut self, s: &str) -> Result {
        for cc in s.chars() {
            ax_console_write_byte(cc as u8);
        }

        Result::Ok(())
    }
}

#[doc(hidden)]
pub fn __print_fmt_sync(args: Arguments) {
    SyncStdout.write_fmt(args).unwrap();
}