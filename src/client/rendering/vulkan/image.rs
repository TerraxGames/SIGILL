//! # Allocated Image
//! A custom image separate from the swapchain.

use ash::{prelude::VkResult, vk};

pub struct AllocatedImage {
    image: super::Image,
    image_view: super::ImageView,
    extent: vk::Extent3D,
    format: vk::Format,
    device: ash::Device,
}

impl AllocatedImage {
    pub(super) fn new(device: &super::Device, image_create_info: &vk::ImageCreateInfo, image_view_create_info: &vk::ImageViewCreateInfo, extent: vk::Extent3D, format: vk::Format) -> VkResult<Self> {
        let image = device.create_image(image_create_info)?;
        let image_view_create_info = image_view_create_info
            .image(*image);
        let image_view = device.create_image_view(&image_view_create_info)?;
        Ok(
            Self {
                image,
                image_view,
                extent,
                format,
                device: device.inner.clone(),
            }
        )
    }

    #[inline]
    pub fn image(&self) -> &super::Image {
        &self.image
    }

    #[inline]
    pub fn image_view(&self) -> &super::ImageView {
        &self.image_view
    }
}
