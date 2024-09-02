//! # Swapchain
//! A collection of utilities for using swapchains.

use ash::{khr, prelude::VkResult, vk};

pub struct Swapchain {
    handle: vk::SwapchainKHR,
    device: khr::swapchain::Device,
    images: Vec<vk::Image>,
    format: vk::Format,
    extent: vk::Extent2D,
}

impl Swapchain {
    pub(super) fn new(handle: vk::SwapchainKHR, device: khr::swapchain::Device, images: Vec<vk::Image>, format: vk::Format, extent: vk::Extent2D) -> Self {
        Self {
            handle,
            device,
            images,
            format,
            extent,
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        // SAFETY: Vulkan functions are available at this time.
        unsafe { self.device.destroy_swapchain(self.handle, None); }
    }
}

pub struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn query(instance: &super::Instance, physical_device: vk::PhysicalDevice) -> VkResult<Self> {
        let surface = instance.surface();
        Ok(
            Self {
                capabilities: instance.get_physical_device_surface_capabilities(physical_device, surface)?,
                formats: instance.get_physical_device_surface_formats(physical_device, surface)?,
                present_modes: instance.get_physical_device_surface_present_modes(physical_device, surface)?,
            }
        )
    }

    pub fn capabilities(&self) -> &vk::SurfaceCapabilitiesKHR {
        &self.capabilities
    }

    pub fn formats(&self) -> &Vec<vk::SurfaceFormatKHR> {
        &self.formats
    }

    pub fn present_modes(&self) -> &Vec<vk::PresentModeKHR> {
        &self.present_modes
    }

    pub fn select_format(&self) -> &vk::SurfaceFormatKHR {
        for available_format in self.formats.iter() {
            if available_format.format == vk::Format::B8G8R8A8_SRGB && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return available_format
            }
        }

        self.formats.get(0).unwrap()
    }

    pub fn select_present_mode(&self, preferred_mode: vk::PresentModeKHR) -> vk::PresentModeKHR {
        for available_present_mode in self.present_modes.iter() {
            if *available_present_mode == preferred_mode {
                return preferred_mode
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn select_extent(&self, width: u32, height: u32) -> vk::Extent2D {
        let capabilities = self.capabilities();
        vk::Extent2D::default()
            .height(height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height))
            .width(width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width))
    }
}
