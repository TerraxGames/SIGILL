use std::ffi::CStr;

use ash::vk;
use thiserror::Error;
use winit::raw_window_handle::{HandleError, HasDisplayHandle};

use crate::*;

pub mod vulkan;

pub struct RenderData {
    pub instance: vulkan::Instance,

    // SAFETY: Keep the `entry` field at the end of the struct declaration so that the `entry` field is dropped last.
    pub entry: ash::Entry,
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
}

pub type RenderResult<T> = Result<T, RenderError>;

pub fn init<T>(app: &mut App, event_loop: &EventLoop<T>) -> Result<(), RenderError> {
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

    // Get window extensions
    let extensions = ash_window::enumerate_required_extensions(event_loop.display_handle()?.as_raw())?;

    // Create instance
    let mut instance_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(extensions);
    if constants::ENABLE_VALIDATION_LAYERS {
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
    let instance = vulkan::Instance::new(&entry, &instance_info)?;

    app.client_data_mut().render_data = Some(RenderData { entry, instance });

    Ok(())
}

pub fn render(app: &mut App) -> Result<(), RenderError> {
    app.window().request_redraw();

    Ok(())
}
