#[cfg(feature = "user")]
use libc;
#[cfg(feature = "user")]
use std::ffi::{CStr, OsStr};
#[cfg(feature = "user")]
use std::os::unix::ffi::OsStrExt;

use crate::core::*;

/// Convert libc::c_chart into a Rust String. Returns an empty string if `ptr` is null or if the
/// String conversion fails.
#[cfg(feature = "user")]
pub unsafe fn to_string(ptr: *const libc::c_char) -> Result<String> {
    if ptr.is_null() {
        Err(UserError::FailedToString.into())
    } else {
        Ok(String::from(OsStr::from_bytes(CStr::from_ptr(ptr).to_bytes()).to_os_string().to_str().ok_or_else(|| UserError::FailedToString)?))
    }
}
