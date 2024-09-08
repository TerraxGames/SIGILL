//! # Vulkan Commands
//! An abstraction for queueing and executing Vulkan commands.

use std::mem::MaybeUninit;

use ash::{prelude::VkResult, vk};

use crate::constants;

/// A collection of a frame's Vulkan commands.
pub struct Frame {
    command_pool_handle: vk::CommandPool,
    command_buffer_handle: vk::CommandBuffer,
    swapchain_semaphore: vk::Semaphore,
    render_semaphore: vk::Semaphore,
    render_fence: vk::Fence,
    device: ash::Device,
}

impl Frame {
    pub(super) fn new(device: ash::Device, command_pool_flags: vk::CommandPoolCreateFlags, queue_family_index: super::QueueFamilyIndex) -> VkResult<Self> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(command_pool_flags)
            .queue_family_index(queue_family_index);
        // SAFETY: The object is automatically destroyed.
        let command_pool_handle = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool_handle.clone())
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);
        // SAFETY: The buffer is automatically destroyed upon its command pool being destroyed.
        let command_buffer_handles = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        let command_buffer_handle = command_buffer_handles[0];
        let semaphore_create_info = vk::SemaphoreCreateInfo::default()
            .flags(vk::SemaphoreCreateFlags::empty());
        // SAFETY: The object is automatically destroyed.
        let swapchain_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        // SAFETY: The object is automatically destroyed.
        let render_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        let fence_create_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);
        // SAFETY: The object is automatically destroyed.
        let render_fence = unsafe { device.create_fence(&fence_create_info, None)? };
        Ok(
            Self {
                command_pool_handle,
                command_buffer_handle,
                swapchain_semaphore,
                render_semaphore,
                render_fence,
                device,
            }
        )
    }

    #[inline]
    pub fn command_buffer_handle(&self) -> vk::CommandBuffer {
        self.command_buffer_handle
    }

    // Command Buffer Management

    /// Wait for rendering to finish.
    #[inline]
    pub fn wait_for_render(&self) -> VkResult<()> {
        // SAFETY: The device is available at this point.
        unsafe {
            self.device.wait_for_fences(&[self.render_fence], true, constants::FENCE_TIMEOUT)?;
            self.device.reset_fences(&[self.render_fence])?;
        }
        Ok(())
    }

    #[inline]
    pub fn swapchain_semaphore(&self) -> vk::Semaphore {
        self.swapchain_semaphore
    }

    #[inline]
    pub fn render_semaphore(&self) -> vk::Semaphore {
        self.render_semaphore
    }

    #[inline]
    pub fn render_fence(&self) -> vk::Fence {
        self.render_fence
    }

    #[inline]
    pub fn reset_command_buffer(&self) -> VkResult<()> {
        // SAFETY: The device is available at this point.
        unsafe { self.device.reset_command_buffer(self.command_buffer_handle, vk::CommandBufferResetFlags::empty()) }
    }

    #[inline]
    pub fn begin_command_buffer(&self, begin_info: vk::CommandBufferBeginInfo) -> VkResult<()> {
        // SAFETY: The device is available at this point.
        unsafe { self.device.begin_command_buffer(self.command_buffer_handle, &begin_info) }
    }

    #[inline]
    pub fn end_command_buffer(&self) -> VkResult<()> {
        // SAFETY: The device is available at this point.
        unsafe { self.device.end_command_buffer(self.command_buffer_handle) }
    }

    // Vulkan Commands

    #[inline]
    pub fn cmd_clear_color_image(&self, image: &super::Image, image_layout: vk::ImageLayout, clear_color_value: vk::ClearColorValue, ranges: &[vk::ImageSubresourceRange]) {
        // SAFETY: The device is available at this point.
        unsafe { self.device.cmd_clear_color_image(self.command_buffer_handle, **image, image_layout, &clear_color_value, ranges); }
    }

    #[inline]
    pub fn cmd_blit_image_2(&self, blit_info: &vk::BlitImageInfo2) {
        // SAFETY: The device is available at this point.
        unsafe { self.device.cmd_blit_image2(self.command_buffer_handle, blit_info) }
    }

    // Utilities

    #[inline]
    pub fn transition_image(&self, image: &super::Image, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) -> VkResult<()> {
        self.transition_image_ex(
            image,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::MEMORY_WRITE,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ,
            old_layout,
            new_layout,
        )
    }

    pub fn transition_image_ex(&self, image: &super::Image, src_stage_mask: vk::PipelineStageFlags2, src_access_mask: vk::AccessFlags2, dst_stage_mask: vk::PipelineStageFlags2, dst_access_mask: vk::AccessFlags2, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) -> VkResult<()> {
        let aspect_flags = if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };
        let subresource_range = super::util::image_subresource_range(aspect_flags);
        let image_barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(src_stage_mask)
            .src_access_mask(src_access_mask)
            .dst_stage_mask(dst_stage_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .subresource_range(subresource_range)
            .image(image.0);
        let image_barriers = [image_barrier];
        let dependency_info = vk::DependencyInfo::default()
            .image_memory_barriers(&image_barriers);
        // SAFETY: The device is available at this point.
        unsafe { self.device.cmd_pipeline_barrier2(self.command_buffer_handle, &dependency_info); }
        Ok(())
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        // SAFETY: The device is available at this point.
        unsafe {
            self.device.destroy_command_pool(self.command_pool_handle, None);
            self.device.destroy_semaphore(self.swapchain_semaphore, None);
            self.device.destroy_semaphore(self.render_semaphore, None);
            self.device.destroy_fence(self.render_fence, None);
        }
    }
}

/// A collection of frames to be rendered.
pub struct Framebuffer {
    frames: [Frame; constants::FRAMEBUFFER_SIZE],
    command_pool_flags: vk::CommandPoolCreateFlags,
    queue_family_index: super::QueueFamilyIndex,
    device: ash::Device,
    current_frame: usize,
}

impl Framebuffer {
    pub(super) fn new(device: &super::Device, command_pool_flags: vk::CommandPoolCreateFlags, queue_family_index: super::QueueFamilyIndex) -> VkResult<Self> {
        Ok(
            Self {
                frames: Framebuffer::_flush(&device.inner, command_pool_flags, queue_family_index)?,
                command_pool_flags,
                queue_family_index,
                device: device.inner.clone(),
                current_frame: 0,
            }
        )
    }

    fn _flush(device: &ash::Device, command_pool_flags: vk::CommandPoolCreateFlags, queue_family_index: super::QueueFamilyIndex) -> VkResult<[Frame; constants::FRAMEBUFFER_SIZE]> {
        let mut frames = [const { MaybeUninit::uninit() }; constants::FRAMEBUFFER_SIZE];
        for (i, elem) in frames.iter_mut().enumerate() {
            // SAFETY: handle errors ourself so that we don't memory leak any already-initialized elements.
            match Frame::new(device.clone(), command_pool_flags, queue_family_index) {
                Ok(frame) => {
                    elem.write(frame);
                },
                Err(e) => {
                    for i in 0..i {
                        unsafe { frames[i].assume_init_drop(); }
                    }

                    return Err(e)
                },
            }
        }
        // SAFETY: The official MaybeUninit docs recommend transmuting an initialized MaybeUninit<T> array to a T array.
        // MaybeUninit has a transparent representation, so this makes sense.
        let frames = unsafe { std::mem::transmute::<_, [Frame; constants::FRAMEBUFFER_SIZE]>(frames) };
        Ok(frames)
    }

    pub fn flush(&mut self) -> VkResult<()> {
        let frames = Framebuffer::_flush(&self.device, self.command_pool_flags, self.queue_family_index)?;
        self.frames = frames;
        Ok(())
    }

    #[inline]
    pub fn current_frame(&self) -> &Frame {
        &self.frames[self.current_frame % constants::FRAMEBUFFER_SIZE]
    }

    #[inline]
    pub fn increment_current_frame(&mut self) {
        self.current_frame += 1;
    }

    #[inline]
    pub fn current_frame_count(&self) -> usize {
        self.current_frame
    }
}
