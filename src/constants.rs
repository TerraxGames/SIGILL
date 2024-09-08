use std::{ffi::{c_char, CString}, sync::LazyLock, time::Duration};

use ash::vk;

// Info
pub const NAME: &'static str = "SIGILL";
pub const C_NAME: LazyLock<CString> = LazyLock::new(|| CString::new(NAME).unwrap());
pub const ISSUE_TRACKER: &'static str = "https://github.com/TerraxGames/SIGILL/issues";
pub const VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
pub const ENGINE_VERSION: u32 = VERSION;
/// The Vulkan API version.
pub const API_VERSION: u32 = vk::API_VERSION_1_3;
pub const API_VERSION_MAJOR: u32 = vk::api_version_major(API_VERSION);
pub const API_VERSION_MINOR: u32 = vk::api_version_minor(API_VERSION);

// Rendering
pub const REQUIRED_VALIDATION_LAYERS: &'static [*const c_char] = &[
    // SAFETY: This is in a 'static lifetime, so the CStr is never freed.
    c"VK_LAYER_KHRONOS_validation".as_ptr()
];
pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
pub const REQUIRED_QUEUE_FAMILIES: LazyLock<vk::QueueFlags> = LazyLock::new(|| vk::QueueFlags::GRAPHICS);
pub const ENABLED_DEVICE_FEATURES: LazyLock<vk::PhysicalDeviceFeatures> = LazyLock::new(||
    vk::PhysicalDeviceFeatures::default()
        .geometry_shader(true)
);
pub const ENABLED_EXTENSIONS: &'static [*const c_char] = &[
    ash::ext::debug_utils::NAME.as_ptr(),
];
pub const ENABLED_DEVICE_EXTENSIONS: &'static [*const c_char] = &[
    ash::khr::swapchain::NAME.as_ptr(),
];
/// A list of queue families used at runtime.
pub const QUEUE_FAMILIES: LazyLock<&'static [vk::QueueFlags]> = LazyLock::new(||
    &[
        vk::QueueFlags::GRAPHICS,
    ]
);
pub const FRAMEBUFFER_SIZE: usize = 2;
pub const FENCE_TIMEOUT: u64 = Duration::from_secs(1).as_nanos() as u64;
pub const MIP_LEVEL: u32 = 0;
pub const SAMPLES: vk::SampleCountFlags = vk::SampleCountFlags::TYPE_1;

// Logging
pub const LOG_LEVEL: log::LevelFilter = {
    if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    }
};
pub const VULKAN_DEBUG_MESSAGE_TYPES: vk::DebugUtilsMessageTypeFlagsEXT = vk::DebugUtilsMessageTypeFlagsEXT::from_raw(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL.as_raw() | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE.as_raw() | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION.as_raw() | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING.as_raw());
