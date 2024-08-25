use std::{ffi::CString, sync::LazyLock};

use ash::vk;

// Info
pub const NAME: &'static str = "SIGILL";
pub const C_NAME: LazyLock<CString> = LazyLock::new(|| CString::new(NAME).unwrap());
pub const ISSUE_TRACKER: &'static str = "https://github.com/TerraxGames/SIGILL/issues";
pub const VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
pub const ENGINE_VERSION: u32 = VERSION;
/// The Vulkan API version.
pub const API_VERSION: u32 = vk::API_VERSION_1_3;

// Logging
pub const LOG_LEVEL: log::LevelFilter = {
    if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    }
};
