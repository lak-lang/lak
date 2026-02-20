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

use std::cmp::Ordering;
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

/// Prints a 32-bit signed integer followed by a newline to stdout.
#[unsafe(no_mangle)]
pub extern "C" fn lak_println_i32(value: i32) {
    println!("{}", value);
}

/// Prints a 64-bit signed integer followed by a newline to stdout.
#[unsafe(no_mangle)]
pub extern "C" fn lak_println_i64(value: i64) {
    println!("{}", value);
}

/// Prints a boolean value followed by a newline to stdout.
///
/// Outputs "true" for `true` and "false" for `false`.
#[unsafe(no_mangle)]
pub extern "C" fn lak_println_bool(value: bool) {
    if value {
        println!("true");
    } else {
        println!("false");
    }
}

/// Compares two C strings for equality.
///
/// Returns `true` if both strings have the same content, `false` otherwise.
/// Handles null pointers: two nulls are equal, a null and non-null are not.
///
/// # Safety
///
/// The caller must ensure that both `a` and `b` are valid null-terminated C strings
/// (or null pointers).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lak_streq(a: *const c_char, b: *const c_char) -> bool {
    if a == b {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    // SAFETY: We verified both pointers are non-null above, and the caller
    // guarantees they point to valid null-terminated C strings.
    let a_str = unsafe { CStr::from_ptr(a) };
    let b_str = unsafe { CStr::from_ptr(b) };
    a_str == b_str
}

/// Compares two C strings lexicographically.
///
/// Returns:
/// - `-1` if `a < b`
/// - `0` if `a == b`
/// - `1` if `a > b`
///
/// Ordering is based on raw byte lexicographical order.
/// For valid UTF-8 strings, this matches Unicode scalar value order.
/// Null pointers are ordered as: null < non-null, and null == null.
///
/// # Safety
///
/// The caller must ensure that both `a` and `b` are valid null-terminated C strings
/// (or null pointers).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lak_strcmp(a: *const c_char, b: *const c_char) -> i32 {
    if a == b {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }

    // SAFETY: We verified both pointers are non-null above, and the caller
    // guarantees they point to valid null-terminated C strings.
    let a_cstr = unsafe { CStr::from_ptr(a) };
    let b_cstr = unsafe { CStr::from_ptr(b) };

    let ordering = a_cstr.cmp(b_cstr);

    match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Prints an error message to stderr and terminates the program with exit code 1.
///
/// This function is called by Lak's `panic()` built-in function.
///
/// # Behavior
///
/// - Non-null, valid UTF-8: prints `panic: {message}\n`
/// - Non-null, invalid UTF-8: prints `panic: {lossy_conversion}\n`
/// - Null pointer: prints `panic: (no message)\n`
///
/// The function never returns. All code paths call `std::process::exit(1)`.
///
/// # Safety
///
/// The caller must ensure that `message` is a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lak_panic(message: *const c_char) -> ! {
    if !message.is_null() {
        // SAFETY: We verified message is non-null, and the caller guarantees
        // it points to a valid null-terminated C string.
        let c_str = unsafe { CStr::from_ptr(message) };
        match c_str.to_str() {
            Ok(rust_str) => eprintln!("panic: {}", rust_str),
            Err(_) => {
                // Invalid UTF-8: print bytes as lossy string
                eprintln!("panic: {}", String::from_utf8_lossy(c_str.to_bytes()));
            }
        }
    } else {
        eprintln!("panic: (no message)");
    }

    std::process::exit(1);
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

    #[test]
    fn test_println_i32() {
        lak_println_i32(42);
        lak_println_i32(-1);
        lak_println_i32(i32::MAX);
        lak_println_i32(i32::MIN);
    }

    #[test]
    fn test_println_i64() {
        lak_println_i64(42);
        lak_println_i64(-1);
        lak_println_i64(i64::MAX);
        lak_println_i64(i64::MIN);
    }

    // lak_streq tests

    #[test]
    fn test_streq_equal_strings() {
        let a = CString::new("hello").unwrap();
        let b = CString::new("hello").unwrap();
        assert!(unsafe { lak_streq(a.as_ptr(), b.as_ptr()) });
    }

    #[test]
    fn test_streq_different_strings() {
        let a = CString::new("hello").unwrap();
        let b = CString::new("world").unwrap();
        assert!(!unsafe { lak_streq(a.as_ptr(), b.as_ptr()) });
    }

    #[test]
    fn test_streq_empty_strings() {
        let a = CString::new("").unwrap();
        let b = CString::new("").unwrap();
        assert!(unsafe { lak_streq(a.as_ptr(), b.as_ptr()) });
    }

    #[test]
    fn test_streq_same_pointer() {
        let a = CString::new("hello").unwrap();
        assert!(unsafe { lak_streq(a.as_ptr(), a.as_ptr()) });
    }

    #[test]
    fn test_streq_both_null() {
        assert!(unsafe { lak_streq(std::ptr::null(), std::ptr::null()) });
    }

    #[test]
    fn test_streq_null_and_non_null() {
        let a = CString::new("hello").unwrap();
        assert!(!unsafe { lak_streq(std::ptr::null(), a.as_ptr()) });
        assert!(!unsafe { lak_streq(a.as_ptr(), std::ptr::null()) });
    }

    #[test]
    fn test_streq_different_length_strings() {
        let a = CString::new("hi").unwrap();
        let b = CString::new("hello").unwrap();
        assert!(!unsafe { lak_streq(a.as_ptr(), b.as_ptr()) });
    }

    #[test]
    fn test_streq_prefix_string() {
        let a = CString::new("hel").unwrap();
        let b = CString::new("hello").unwrap();
        assert!(!unsafe { lak_streq(a.as_ptr(), b.as_ptr()) });
    }

    // lak_strcmp tests

    #[test]
    fn test_strcmp_equal_strings() {
        let a = CString::new("hello").unwrap();
        let b = CString::new("hello").unwrap();
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), b.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_less_than() {
        let a = CString::new("apple").unwrap();
        let b = CString::new("banana").unwrap();
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), b.as_ptr()) }, -1);
    }

    #[test]
    fn test_strcmp_greater_than() {
        let a = CString::new("banana").unwrap();
        let b = CString::new("apple").unwrap();
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), b.as_ptr()) }, 1);
    }

    #[test]
    fn test_strcmp_lexicographical_numeric_text() {
        let a = CString::new("z").unwrap();
        let b = CString::new("10").unwrap();
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), b.as_ptr()) }, 1);
    }

    #[test]
    fn test_strcmp_empty_and_non_empty() {
        let a = CString::new("").unwrap();
        let b = CString::new("a").unwrap();
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), b.as_ptr()) }, -1);
    }

    #[test]
    fn test_strcmp_prefix_string() {
        let a = CString::new("hel").unwrap();
        let b = CString::new("hello").unwrap();
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), b.as_ptr()) }, -1);
    }

    #[test]
    fn test_strcmp_null_handling() {
        let a = CString::new("hello").unwrap();
        assert_eq!(unsafe { lak_strcmp(std::ptr::null(), std::ptr::null()) }, 0);
        assert_eq!(unsafe { lak_strcmp(std::ptr::null(), a.as_ptr()) }, -1);
        assert_eq!(unsafe { lak_strcmp(a.as_ptr(), std::ptr::null()) }, 1);
    }
}
