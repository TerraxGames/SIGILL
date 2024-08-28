use std::ffi::c_void;

use ash::vk;

use crate::constants;

use super::{vulkan, RenderResult};

pub fn init_vulkan_debug_callback(instance: &vulkan::Instance) -> RenderResult<vulkan::DebugUtilsMessenger> {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE | vk::DebugUtilsMessageSeverityFlagsEXT::INFO | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
        .message_type(constants::VULKAN_DEBUG_MESSAGE_TYPES)
        .pfn_user_callback(Some(vulkan_debug_callback));
    instance.create_debug_utils_messenger_ext(&create_info)
}

unsafe extern "system" fn vulkan_debug_callback(
    severity_flags: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type_flags: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    // sanity check just in case
    if callback_data.is_null() {
        return vk::FALSE
    }
    // SAFETY: this dereference is null-checked.
    let callback_data = unsafe { *callback_data };

    let severity = severity_from_flags(&severity_flags);
    // Don't report severity levels higher than allowed.
    if severity > constants::LOG_LEVEL {
        return vk::FALSE
    }

    let ((Some(message), _) | (None, message)) = (unsafe { callback_data.message_as_c_str() }, c"<no message>");
    let message = message.to_string_lossy().to_string();
    log::log!(target: "Vulkan", severity, "{message}");
    
    vk::FALSE
}

fn severity_from_flags(severity_flags: &vk::DebugUtilsMessageSeverityFlagsEXT) -> log::Level {
    match *severity_flags {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::Level::Trace,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::Level::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::Level::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::Level::Error,
        _ => panic!("Unsupported Vulkan severity flags: {severity_flags:?}"),
    }
}
