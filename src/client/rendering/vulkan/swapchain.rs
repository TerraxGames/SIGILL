//! # Swapchain
//! A collection of utilities for using swapchains.

use ash::{khr, prelude::VkResult, vk};

use crate::constants;

pub struct Swapchain {
    handle: vk::SwapchainKHR,
    device: khr::swapchain::Device,
    images: Vec<super::Image>,
    image_view: Vec<super::ImageView>,
    format: vk::Format,
    extent: vk::Extent3D,
}

impl Swapchain {
    pub(super) fn new(handle: vk::SwapchainKHR, device: khr::swapchain::Device, images: Vec<super::Image>, image_view: Vec<super::ImageView>, format: vk::Format, extent: vk::Extent3D) -> Self {
        Self {
            handle,
            device,
            images,
            image_view,
            format,
            extent,
        }
    }

    #[inline]
    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }
    
    #[inline]
    pub fn extent(&self) -> vk::Extent3D {
        self.extent
    }

    #[inline]
    pub fn acquire_next_image(&self, frame: &super::commands::Frame) -> VkResult<u32> {
        // SAFETY: The device is available at this point.
        Ok(
            unsafe { self.device.acquire_next_image(self.handle, constants::FENCE_TIMEOUT, frame.swapchain_semaphore(), vk::Fence::null())?.0 }
        )
    }

    #[inline]
    pub fn get_image(&self, image_index: u32) -> Option<&super::Image> {
        self.images.get(image_index as usize)
    }

    #[inline]
    pub fn present_queue<'a>(&self, queue: &super::queues::Queue, present_info: &'a vk::PresentInfoKHR<'a>) -> VkResult<bool> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.device.queue_present(queue.handle(), present_info) }
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
