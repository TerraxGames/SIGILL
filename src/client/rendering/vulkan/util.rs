use ash::vk;

use crate::constants;

/// metaphorically "memcpy"s an image to another image.
/// i have nothing better to call this i promise.
pub fn memcpy_image(frame: &super::commands::Frame, src: &super::Image, dst: &super::Image, src_size: vk::Extent3D, dst_size: vk::Extent3D, src_subresource: vk::ImageSubresourceLayers, dst_subresource: vk::ImageSubresourceLayers) {
    let blit_region = vk::ImageBlit2::default()
        .src_offsets(
            [
                Default::default(),
                vk::Offset3D::default()
                    .x(src_size.width as i32)
                    .y(src_size.height as i32)
                    .z(src_size.depth as i32),
            ],
        )
        .dst_offsets(
            [
                Default::default(),
                vk::Offset3D::default()
                    .x(dst_size.width as i32)
                    .y(dst_size.height as i32)
                    .z(dst_size.depth as i32),
            ],
        )
        .src_subresource(src_subresource)
        .dst_subresource(dst_subresource);
    let blit_info = vk::BlitImageInfo2::default()
        .src_image(**src)
        .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .dst_image(**dst)
        .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .filter(vk::Filter::LINEAR)
        .regions(std::slice::from_ref(&blit_region));
    frame.cmd_blit_image_2(&blit_info);
}

// Info Structs

#[inline]
pub fn image_subresource_range(aspect_flags: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
            .aspect_mask(aspect_flags)
            .base_mip_level(0)
            .level_count(vk::REMAINING_MIP_LEVELS)
            .base_array_layer(0)
            .layer_count(vk::REMAINING_ARRAY_LAYERS)
}

#[inline]
pub fn semaphore_submit_info<'a>(stage_mask: vk::PipelineStageFlags2, semaphore: vk::Semaphore) -> vk::SemaphoreSubmitInfo<'a> {
    semaphore_submit_info_ex(stage_mask, semaphore, 0, 1)
}

#[inline]
pub fn semaphore_submit_info_ex<'a>(stage_mask: vk::PipelineStageFlags2, semaphore: vk::Semaphore, device_index: u32, value: u64) -> vk::SemaphoreSubmitInfo<'a> {
    vk::SemaphoreSubmitInfo::default()
        .semaphore(semaphore)
        .stage_mask(stage_mask)
        .device_index(device_index)
        .value(value)
}

#[inline]
pub fn command_buffer_submit_info<'a>(command_buffer: vk::CommandBuffer) -> vk::CommandBufferSubmitInfo<'a> {
    command_buffer_submit_info_ex(command_buffer, 0)
}

#[inline]
pub fn command_buffer_submit_info_ex<'a>(command_buffer: vk::CommandBuffer, device_mask: u32) -> vk::CommandBufferSubmitInfo<'a> {
    vk::CommandBufferSubmitInfo::default()
        .command_buffer(command_buffer)
        .device_mask(device_mask)
}

#[inline]
pub fn submit_info<'a>(command_buffer_submit_info: &'a vk::CommandBufferSubmitInfo<'a>, signal_semaphore_submit_info: &'a Option<vk::SemaphoreSubmitInfo<'a>>, wait_semaphore_submit_info: &'a Option<vk::SemaphoreSubmitInfo<'a>>) -> vk::SubmitInfo2<'a> {
    submit_info_ex(
        std::slice::from_ref(command_buffer_submit_info),
        signal_semaphore_submit_info.as_ref().map(std::slice::from_ref).unwrap_or_default(),
        wait_semaphore_submit_info.as_ref().map(std::slice::from_ref).unwrap_or_default(),
    )
}

#[inline]
pub fn submit_info_ex<'a>(command_buffer_submit_infos: &'a [vk::CommandBufferSubmitInfo<'a>], signal_semaphore_submit_infos: &'a [vk::SemaphoreSubmitInfo<'a>], wait_semaphore_submit_infos: &'a [vk::SemaphoreSubmitInfo<'a>]) -> vk::SubmitInfo2<'a> {
    vk::SubmitInfo2::default()
        .wait_semaphore_infos(wait_semaphore_submit_infos)
        .signal_semaphore_infos(signal_semaphore_submit_infos)
        .command_buffer_infos(command_buffer_submit_infos)
}

#[inline]
pub fn image_info_2d<'a>(format: vk::Format, extent: vk::Extent2D, image_usage_flags: vk::ImageUsageFlags) -> vk::ImageCreateInfo<'a> {
    image_info_ex(
        format,
        extent.into(),
        vk::ImageType::TYPE_2D,
        constants::MIP_LEVELS,
        constants::SAMPLES,
        image_usage_flags,
    )
}

#[inline]
pub fn image_info_ex<'a>(format: vk::Format, extent: vk::Extent3D, image_type: vk::ImageType, mip_levels: u32, samples: vk::SampleCountFlags, image_usage_flags: vk::ImageUsageFlags) -> vk::ImageCreateInfo<'a> {
    vk::ImageCreateInfo::default()
        .image_type(image_type)
        .format(format)
        .extent(extent)
        .mip_levels(mip_levels)
        .array_layers(1)
        .samples(samples)
        .tiling(vk::ImageTiling::OPTIMAL) // always use the optimal format, for performance
        .usage(image_usage_flags)
}

#[inline]
pub fn image_view_create_info_2d<'a>(format: vk::Format, image: Option<&super::Image>, image_aspect_flags: vk::ImageAspectFlags) -> vk::ImageViewCreateInfo<'a> {
    image_view_create_info_ex(
        vk::ImageViewType::TYPE_2D,
        format,
        image,
        vk::ImageSubresourceRange::default()
            .base_mip_level(0)
            .level_count(constants::MIP_LEVELS)
            .base_array_layer(0)
            .layer_count(1)
            .aspect_mask(image_aspect_flags),
    )
}

#[inline]
pub fn image_view_create_info_ex<'a>(image_view_type: vk::ImageViewType, format: vk::Format, image: Option<&super::Image>, subresource_range: vk::ImageSubresourceRange) -> vk::ImageViewCreateInfo<'a> {
    let mut create_info = vk::ImageViewCreateInfo::default()
        .view_type(image_view_type)
        .format(format)
        .subresource_range(subresource_range);
    if let Some(image) = image {
        create_info = create_info.image(**image);
    }

    create_info
}
