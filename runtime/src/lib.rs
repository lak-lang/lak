//! Lak runtime library.
//!
//! Provides runtime functions called by compiled Lak programs.
//! This library is compiled as a static library (`staticlib`) and linked
//! with generated object files to produce final executables.
//!
//! # ABI
//!
//! All exported functions use the C calling convention (`extern "C"`)
//! to ensure compatibility with LLVM-generated code.

use std::ffi::CStr;
use std::os::raw::c_char;

/// Prints a string followed by a newline to stdout.
///
/// # Safety
///
/// The caller must ensure that `s` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lak_println(s: *const c_char) {
    if s.is_null() {
        println!();
        return;
    }

    // SAFETY: We verified s is non-null above, and the caller guarantees
    // it points to a valid null-terminated C string.
    let c_str = unsafe { CStr::from_ptr(s) };
    match c_str.to_str() {
        Ok(rust_str) => println!("{}", rust_str),
        Err(_) => {
            // Invalid UTF-8: print bytes as lossy string
            println!("{}", String::from_utf8_lossy(c_str.to_bytes()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_null_pointer() {
        // Should print empty line, not crash
        unsafe { lak_println(std::ptr::null()) };
    }

    #[test]
    fn test_valid_string() {
        let s = CString::new("Hello, World!").unwrap();
        unsafe { lak_println(s.as_ptr()) };
    }

    #[test]
    fn test_empty_string() {
        let s = CString::new("").unwrap();
        unsafe { lak_println(s.as_ptr()) };
    }

    #[test]
    fn test_unicode() {
        let s = CString::new("こんにちは世界").unwrap();
        unsafe { lak_println(s.as_ptr()) };
    }

    #[test]
    fn test_escape_sequences() {
        let s = CString::new("hello\tworld\n").unwrap();
        unsafe { lak_println(s.as_ptr()) };
    }
}
