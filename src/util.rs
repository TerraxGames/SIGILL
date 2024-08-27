#[macro_export]
macro_rules! cstr {
    ( $string:literal ) => {
        unsafe { use core::ffi::CStr; CStr::from_bytes_with_nul_unchecked(b"$string\0") }
    };
}
