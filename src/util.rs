#[macro_export]
// SAFETY: The string always contains a null byte.
macro_rules! cstr {
    ( $string:literal ) => {
        unsafe { use core::ffi::CStr; CStr::from_bytes_with_nul_unchecked(b"$string\0") }
    };
}
