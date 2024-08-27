use std::{ffi::{c_char, CStr, CString}, ptr::slice_from_raw_parts, sync::LazyLock};

use ash::vk;

// Info
pub const NAME: &'static str = "SIGILL";
pub const C_NAME: LazyLock<CString> = LazyLock::new(|| CString::new(NAME).unwrap());
pub const ISSUE_TRACKER: &'static str = "https://github.com/TerraxGames/SIGILL/issues";
pub const VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
pub const ENGINE_VERSION: u32 = VERSION;
/// The Vulkan API version.
pub const API_VERSION: u32 = vk::API_VERSION_1_3;

// Rendering
pub const REQUIRED_VALIDATION_LAYERS: &'static [*const c_char] = &[
    // SAFETY: This is in a 'static lifetime, so the CStr is never freed.
    c"VK_LAYER_KHRONOS_validation".as_ptr()
];
pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);

// Logging
pub const LOG_LEVEL: log::LevelFilter = {
    if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    }
};
