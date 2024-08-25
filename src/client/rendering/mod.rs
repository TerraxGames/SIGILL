use ash::vk;
use constants::C_NAME;
use thiserror::Error;
use crate::util::VulkanObject;
use winit::raw_window_handle::{HandleError, HasDisplayHandle};

use crate::*;

pub struct RenderData {
    pub entry: ash::Entry,
    pub instance: VulkanObject<ash::Instance>,
}

impl Drop for RenderData {
    fn drop(&mut self) {
        self.instance.drop();
    }
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("error initializing renderer: {0}")]
    LoadingError(#[from] ash::LoadingError),
    #[error("Vulkan error: {0}")]
    VkResult(#[from] vk::Result),
    #[error("error obtaining handle: {0}")]
    HandleError(#[from] HandleError),
}

pub fn init<T>(app: &mut App, event_loop: &EventLoop<T>) -> Result<(), RenderError> {
    let entry = unsafe { ash::Entry::load()? };

    let extensions = unsafe { entry.enumerate_instance_extension_properties(None)? };
    debug!("{} extensions supported", extensions.len());
    
    let app_name = &*C_NAME;
    let app_info = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(constants::VERSION)
        .engine_name(app_name)
        .engine_version(constants::ENGINE_VERSION)
        .api_version(constants::API_VERSION);

    // Get window extensions
    let extensions = ash_window::enumerate_required_extensions(event_loop.display_handle()?.as_raw())?;

    // Create instance
    let instance_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(extensions);
    let instance: VulkanObject<ash::Instance> = VulkanObject::new(unsafe { entry.create_instance(&instance_info, None)? }, drop::instance);

    app.client_data_mut().render_data = Some(RenderData { entry, instance });

    Ok(())
}

pub fn render(app: &mut App) -> Result<(), RenderError> {
    app.window().request_redraw();

    Ok(())
}
