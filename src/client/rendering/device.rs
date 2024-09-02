//! # Physical Device Selection
//! This module provides utilities for selecting and ranking physical devices.

use std::{collections::HashSet, ffi::CStr, hash::RandomState};

use ash::vk::{self, QueueFlags};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::{constants, vulkan, App, RenderError, RenderResult};

pub struct RankedDevice(u32, vk::PhysicalDevice);

impl PartialEq for RankedDevice {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RankedDevice {}

impl PartialOrd for RankedDevice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for RankedDevice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// Select the most suitable device for rendering.
pub fn find_suitable_device<'a>(instance: &mut vulkan::Instance, app: &App) -> RenderResult<(vk::PhysicalDevice, vulkan::SwapchainSupport)> {
    let physical_devices = instance.enumerate_physical_devices()?;
    for physical_device in physical_devices.iter() {
        let supported = check_device_capabilities(instance, *physical_device, app).expect("failed to check device capabilities");
        if supported {
            break
        }
    }

    let mut physical_devices = physical_devices
        .into_iter()
        .map(|physical_device| RankedDevice(rank_device_capabilities(&instance, physical_device), physical_device))
        .collect::<Vec<RankedDevice>>();
    physical_devices.sort();

    let suitable_device = physical_devices.last();
    if let Some(suitable_device) = suitable_device {
        let suitable_device = suitable_device.1;
        instance.create_surface(app.window().display_handle()?.as_raw(), app.window().window_handle()?.as_raw())?;
        let swapchain_support = vulkan::SwapchainSupport::query(&instance, suitable_device)?;

        return Ok((suitable_device, swapchain_support))
    } else {
        return Err(RenderError::UnsupportedDevice)
    }
}

/// Ensures that the device meets basic requirements.
pub fn check_device_capabilities(instance: &mut vulkan::Instance, physical_device: vk::PhysicalDevice, app: &App) -> RenderResult<bool> {
    let properties = instance.get_physical_device_properties(physical_device);
    let supported_gpu = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU || properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU;
    let supports_vulkan_version = vk::api_version_major(properties.api_version) == constants::API_VERSION_MAJOR || vk::api_version_minor(properties.api_version) >= constants::API_VERSION_MINOR;

    let features = instance.get_physical_device_features(physical_device);
    let supports_geometry_shader = features.geometry_shader == vk::TRUE;
    let supports_required_features = supports_geometry_shader;

    let mut available_queue_families = QueueFlags::empty();
    let queue_families = instance.get_physical_device_queue_family_properties(physical_device);
    for queue_family in queue_families.iter() {
        available_queue_families |= queue_family.queue_flags;
    }
    let has_required_queue_families = available_queue_families.contains(*constants::REQUIRED_QUEUE_FAMILIES);

    let available_extensions = instance.enumerate_device_extension_properties(physical_device)?;
    let mut required_extensions: HashSet<String, RandomState> = HashSet::from_iter(constants::ENABLED_EXTENSIONS.iter().map(|&ptr| {
        // SAFETY: The extension names are guaranteed to be valid C strings.
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    }));
    required_extensions.extend(constants::ENABLED_DEVICE_EXTENSIONS.iter().map(|&ptr| {
        // SAFETY: The extension names are guaranteed to be valid C strings.
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    }));
    for available_extension in available_extensions {
        // SAFETY: The extension names are guaranteed to be valid C strings.
        let extension_name = unsafe { CStr::from_ptr(available_extension.extension_name.as_ptr()) }.to_string_lossy().to_string();
        required_extensions.remove(&extension_name);
    }
    let supports_required_extensions = required_extensions.is_empty();

    // Verify surface capabilities.
    instance.create_surface(app.window().display_handle()?.as_raw(), app.window().window_handle()?.as_raw())?;
    let swap_chain_support = vulkan::SwapchainSupport::query(&instance, physical_device)?;
    let swap_chain_adequate = !swap_chain_support.formats().is_empty() && !swap_chain_support.present_modes().is_empty();
    
    Ok(supported_gpu && supports_vulkan_version && supports_required_features && has_required_queue_families && supports_required_extensions && swap_chain_adequate)
}

/// Rank the device based on its capabilities.
pub fn rank_device_capabilities(instance: &vulkan::Instance, physical_device: vk::PhysicalDevice) -> u32 {
    let mut score = 0u32;

    let properties = instance.get_physical_device_properties(physical_device);
    // Prefer dedicated GPUs
    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 1000;
    }
    // Prefer higher maximum image dimensions since those affect graphics quality.
    score += properties.limits.max_image_dimension2_d;

    score
}
