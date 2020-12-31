use crate::{core::*, errors::*};
use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
};

/// Convert libc::c_chart into a Rust String. Returns an empty string if `ptr` is null or if the
/// String conversion fails.
///
/// # Safety
/// FFI with libc
pub unsafe fn to_string(ptr: *const libc::c_char) -> FuResult<String> {
    if ptr.is_null() {
        Err(StringError::FailedToString.into())
    } else {
        OsStr::from_bytes(CStr::from_ptr(ptr).to_bytes()).to_string()
    }
}
