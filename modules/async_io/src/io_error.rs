use alloc::{format, string::String};
use axerrno::AxError;
use core::error::Error;
use core::fmt;

#[derive(Debug)]
pub struct IoError {
    kind: AxError,
    err_msg: String,
}

impl IoError {
    pub fn new(kind: AxError, err_msg: String) -> Self {
        Self { kind, err_msg }
    }
}

impl From<AxError> for IoError {
    fn from(value: AxError) -> Self {
        Self {
            kind: value,
            err_msg: format!("{:?}", value),
        }
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IoError: {} ({})", self.err_msg, self.kind)
    }
}

impl Error for IoError {
    /* 这边不知道咋整，先全默认好了 */
}
