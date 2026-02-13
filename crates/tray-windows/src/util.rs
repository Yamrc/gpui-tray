//! Windows string utilities

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

/// Convert a string to wide characters (UTF-16)
pub fn encode_wide<S: AsRef<OsStr>>(string: S) -> Vec<u16> {
    OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}
