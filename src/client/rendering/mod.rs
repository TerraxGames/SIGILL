use std::{ffi::CStr, ops::Deref};

use ash::vk;
use thiserror::Error;
use winit::{event_loop::ActiveEventLoop, raw_window_handle::{HandleError, HasDisplayHandle}};

use crate::*;

pub mod vulkan;
pub mod log;
pub mod device;
pub mod queues;

#[allow(unused)]
pub struct RenderData {
    pub queue_families: queues::QueueFamilies,
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
}

pub type RenderResult<T> = Result<T, RenderError>;

pub fn init(app: &mut App, event_loop: &ActiveEventLoop) -> Result<(), RenderError> {
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
    let mut queue_families = queues::QueueFamilies::new_empty(&queue_family_map);
    queue_families = queue_families.query_present_mode_queue(&queue_family_map, &instance, selected_physical_device, instance.surface())?;
    trace!("Using Queue Families: {queue_families:#?}");

    // Create swapchain info.
    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(*instance.surface().deref())
        .min_image_count(capabilities.min_image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(swapchain_support.select_extent(app.window().inner_size().width, app.window().inner_size().height))
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
    let queue_family_indices = vec![queue_families.graphics().queue_info().0, queue_families.present_mode().queue_info().0];

    if queue_families.graphics().queue_info() != queue_families.present_mode().queue_info() {
        swapchain_create_info = swapchain_create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(queue_family_indices.as_slice());
    } else {
        swapchain_create_info = swapchain_create_info
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE);
    }

    swapchain_create_info = swapchain_create_info
        .pre_transform(swapchain_support.capabilities().current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(swapchain_support.select_present_mode(vk::PresentModeKHR::MAILBOX));

    // Get queue creation info.
    let queue_create_infos = queue_families.get_queue_create_infos(&queue_family_map);
    trace!("Queue Creation Info: {queue_create_infos:?}");

    // Create device.
    let enabled_device_features = &*constants::ENABLED_DEVICE_FEATURES;
    // don't enable device-specific layers because we don't support shitty Vulkan implementations
    let device_create_info = vk::DeviceCreateInfo::default()
        .enabled_features(enabled_device_features)
        .enabled_extension_names(constants::ENABLED_DEVICE_EXTENSIONS)
        .queue_create_infos(queue_create_infos.as_slice());
    instance.create_device(selected_physical_device, &device_create_info)?;

    // Create swapchain.
    instance.create_swapchain(&swapchain_create_info)?;

    // Populate Queue handles.
    queue_families.populate_handles(instance.device());

    app.client_data_mut().render_data = Some(RenderData {
        queue_families,
        selected_physical_device,
        instance,
    });

    Ok(())
}

pub fn render(app: &mut App) -> Result<(), RenderError> {
    app.window().request_redraw();

    Ok(())
}
