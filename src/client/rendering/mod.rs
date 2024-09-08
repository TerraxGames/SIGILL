use std::{ffi::CStr, ops::Deref};

use ash::vk;
use thiserror::Error;
use winit::{event_loop::ActiveEventLoop, raw_window_handle::{HandleError, HasDisplayHandle}};

use crate::*;

pub mod vulkan;
pub mod log;
pub mod device;

#[allow(unused)]
pub struct RenderData {
    pub queue_families: vulkan::queues::QueueFamilies,
    pub selected_physical_device: vk::PhysicalDevice,
    pub instance: vulkan::Instance,
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("error initializing renderer: {0}")]
    LoadingError(#[from] ash::LoadingError),
    #[error("Vulkan error: {0}")]
    VkResult(#[from] vk::Result),
    #[error("error obtaining handle: {0}")]
    HandleError(#[from] HandleError),
    #[error("validation layer not found: {0}")]
    ValidationLayerNotFound(String),
    #[error("no supported graphics devices were found")]
    UnsupportedDevice,
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type RenderResult<T> = Result<T, RenderError>;

pub fn init(app: &mut App, event_loop: &ActiveEventLoop) -> RenderResult<()> {
    warn!("Now loading Vulkan library. If the game crashes after this warning, check to see if your system supports Vulkan!");
    // SAFETY: ¯\_(ツ)_/¯
    // Beware of garbage error messages on UNIX-likes, since `dlerror` is not MT-safe.
    // Also, DO NOT modify the DLL path during initialization.
    // Do not multi-thread until rendering has initialized.
    let entry = unsafe { ash::Entry::load()? };
    info!("Vulkan has loaded.");
    
    let app_name = &*constants::C_NAME;
    let app_info = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(constants::VERSION)
        .engine_name(app_name)
        .engine_version(constants::ENGINE_VERSION)
        .api_version(constants::API_VERSION);

    // Get required extensions
    let mut extensions = ash_window::enumerate_required_extensions(event_loop.display_handle()?.as_raw())?.to_vec();
    extensions.extend_from_slice(constants::ENABLED_EXTENSIONS);

    // Create instance
    let mut instance_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extensions);
    if constants::ENABLE_VALIDATION_LAYERS {
        // Ensure the required validation layers are available.
        let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };
        
        for required_validation_layer_bytes in constants::REQUIRED_VALIDATION_LAYERS {
            // SAFETY: This is always a valid CStr.
            let required_validation_layer = unsafe { CStr::from_ptr(*required_validation_layer_bytes) };

            if available_layers.iter().find(|layer| {
                layer.layer_name_as_c_str().unwrap().eq(required_validation_layer)
            }).is_none() {
                return Err(RenderError::ValidationLayerNotFound(required_validation_layer.to_string_lossy().to_string()))
            }
        }
        
        instance_info = instance_info.enabled_layer_names(constants::REQUIRED_VALIDATION_LAYERS);
    }
    let mut instance = vulkan::Instance::new(entry, &instance_info)?;

    if cfg!(debug_assertions) {
        // Set up debugging
        log::init_vulkan_debug_callback(&mut instance)?;
    }

    // Find a suitable physical device and create window surface.
    let (selected_physical_device, swapchain_support) = device::find_suitable_device(&mut instance, app)?;

    // Extract swapchain capabilities.
    let capabilities = swapchain_support.capabilities();
    let format = swapchain_support.select_format();

    // Get queue families for use during device creation.
    let queue_flags = *constants::QUEUE_FAMILIES;
    let queue_family_map = instance.get_queue_family_map(selected_physical_device, queue_flags);
    debug!("Queue Families queried: {queue_family_map:?}");
    let mut queue_families = vulkan::queues::QueueFamilies::new_empty(&queue_family_map);
    queue_families = queue_families.query_present_mode_queue(&queue_family_map, &instance, selected_physical_device, instance.surface())?;
    trace!("Using Queue Families: {queue_families:#?}");

    // Create swapchain info.
    let image_extent = swapchain_support.select_extent(app.window().inner_size().width, app.window().inner_size().height);
    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(*instance.surface().deref())
        .min_image_count(capabilities.min_image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(image_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST);
    let queue_family_indices = vec![queue_families.graphics().queue_info().0, queue_families.present_mode().queue_info().0];

    if queue_families.graphics().queue_info() != queue_families.present_mode().queue_info() {
        swapchain_create_info = swapchain_create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(queue_family_indices.as_slice());
    } else {
        swapchain_create_info = swapchain_create_info
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE);
    }

    let present_mode = swapchain_support.select_present_mode(vk::PresentModeKHR::MAILBOX);
    trace!("Present mode: {present_mode:?}");
    swapchain_create_info = swapchain_create_info
        .pre_transform(swapchain_support.capabilities().current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode);

    // Get queue creation info.
    let queue_create_infos = queue_families.get_queue_create_infos(&queue_family_map);
    trace!("Queue Creation Info: {queue_create_infos:?}");

    // Enable special Synchronization2 feature.
    let mut synchronization2_feature = vk::PhysicalDeviceSynchronization2Features::default()
        .synchronization2(true);
    // Create device.
    let enabled_device_features = &*constants::ENABLED_DEVICE_FEATURES;
    // don't enable device-specific layers because we don't support shitty Vulkan implementations
    let device_create_info = vk::DeviceCreateInfo::default()
        .enabled_features(enabled_device_features)
        .enabled_extension_names(constants::ENABLED_DEVICE_EXTENSIONS)
        .queue_create_infos(queue_create_infos.as_slice())
        .push_next(&mut synchronization2_feature);
    instance.create_device(selected_physical_device, &device_create_info)?;

    // Create swapchain.
    instance.create_swapchain(
        &swapchain_create_info,
        |images, format| {
            Vec::from_iter(
                images
                    .iter()
                    .map(|image| {
                        vk::ImageViewCreateInfo::default()
                            .image(**image)
                            .format(format)
                            .view_type(vk::ImageViewType::TYPE_2D)
                            .components(
                                vk::ComponentMapping::default()
                                    .r(vk::ComponentSwizzle::IDENTITY)
                                    .g(vk::ComponentSwizzle::IDENTITY)
                                    .b(vk::ComponentSwizzle::IDENTITY)
                                    .a(vk::ComponentSwizzle::IDENTITY)
                            )
                            .subresource_range(
                                vulkan::util::image_subresource_range(vk::ImageAspectFlags::COLOR)
                            )
                    })
            )
        },
    )?;

    // Populate Queue handles.
    queue_families.populate_handles(instance.device());

    instance.create_framebuffer(
        vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_families.graphics().queue_info().0,
    )?;

    let draw_image_format = vk::Format::R16G16B16A16_SFLOAT;
    let draw_image_extent = image_extent;
    let mut draw_image_usages = vk::ImageUsageFlags::empty();
    draw_image_usages |= vk::ImageUsageFlags::TRANSFER_SRC;
    draw_image_usages |= vk::ImageUsageFlags::TRANSFER_DST;
    draw_image_usages |= vk::ImageUsageFlags::STORAGE;
    draw_image_usages |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    let draw_image_info = vulkan::util::image_info_2d(draw_image_format, draw_image_extent, draw_image_usages);
    let draw_image_view_info = vulkan::util::image_view_create_info_2d(draw_image_format, None, vk::ImageAspectFlags::COLOR);
    instance.create_draw_image(&draw_image_info, &draw_image_view_info, draw_image_extent.into(), draw_image_format)?;

    app.client_data_mut().render_data = Some(RenderData {
        queue_families,
        selected_physical_device,
        instance,
    });

    Ok(())
}

pub fn begin_render(app: &mut App) -> RenderResult<()> {
    app.window().request_redraw();

    let render_data = app.render_data_mut();
    let instance = &mut render_data.instance;
    let current_frame = instance.framebuffer().current_frame();
    // Wait until the GPU has finished rendering the last frame.
    current_frame.wait_for_render()?;

    // Prepare command buffer.
    let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    current_frame.reset_command_buffer()?;
    current_frame.begin_command_buffer(command_buffer_begin_info)?;
    current_frame.transition_image(instance.draw_image().image(), vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL)?;

    Ok(())
}

pub fn render_background(app: &mut App) -> RenderResult<()> {
    let render_data = app.render_data_mut();
    let instance = &mut render_data.instance;
    let current_frame = instance.framebuffer().current_frame();

    // Draw flashing color.
    let flash = f32::abs(f32::sin(std::f32::consts::PI * instance.framebuffer().current_frame_count() as f32 / (144.0 * 16.0)));
    let clear_color = vk::ClearColorValue {
        float32: [0.2 * flash, 0.25 * flash, flash, 1.0],
    };
    let clear_range = vulkan::util::image_subresource_range(vk::ImageAspectFlags::COLOR);
    current_frame.cmd_clear_color_image(instance.draw_image().image(), vk::ImageLayout::GENERAL, clear_color, &[clear_range]);

    Ok(())
}

pub fn end_render(app: &mut App) -> RenderResult<()> {
    let render_data = app.render_data_mut();
    let instance = &mut render_data.instance;
    let current_frame = instance.framebuffer().current_frame();

    // Request image from the swapchain.
    let swapchain = instance.swapchain();
    let swapchain_image_index = swapchain.acquire_next_image(current_frame)?;
    let swapchain_image = swapchain.get_image(swapchain_image_index).expect("image should have been present in swapchain");

    // Transition draw image back, copy it to the swapchain image, and end command buffer.
    current_frame.transition_image(instance.draw_image().image(), vk::ImageLayout::GENERAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL)?;
    current_frame.transition_image(swapchain_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL)?;
    let image_subresource_layers = vulkan::util::image_subresource_layers(vk::ImageAspectFlags::COLOR);
    vulkan::util::memcpy_image(current_frame, instance.draw_image().image(), swapchain_image, instance.draw_image().extent(), swapchain.extent(), image_subresource_layers, image_subresource_layers);
    current_frame.transition_image(swapchain_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR)?;
    current_frame.end_command_buffer()?;

    // Prepare queue submission.
    let command_buffer_submit_info = vulkan::util::command_buffer_submit_info(current_frame.command_buffer_handle());
    let wait_semaphore_submit_info = Some(vulkan::util::semaphore_submit_info(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, current_frame.swapchain_semaphore()));
    let signal_semaphore_submit_info = Some(vulkan::util::semaphore_submit_info(vk::PipelineStageFlags2::ALL_GRAPHICS, current_frame.render_semaphore()));
    let submit_info = vulkan::util::submit_info(&command_buffer_submit_info, &signal_semaphore_submit_info, &wait_semaphore_submit_info);
    
    render_data.queue_families.submit_queue(instance.device(), vulkan::queues::QueueType::Graphics, &submit_info, current_frame.render_fence())?;

    let swapchain_handle = swapchain.handle();
    let render_semaphore = current_frame.render_semaphore();
    let present_info = vk::PresentInfoKHR::default()
        .swapchains(std::slice::from_ref(&swapchain_handle))
        .wait_semaphores(std::slice::from_ref(&render_semaphore))
        .image_indices(std::slice::from_ref(&swapchain_image_index));

    swapchain.present_queue(render_data.queue_families.graphics(), &present_info)?;

    instance.framebuffer_mut().increment_current_frame();

    Ok(())
}
