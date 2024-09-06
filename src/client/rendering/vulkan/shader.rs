//! # Shader Abstractions
//! Abstractions for opening and loading SPIR-V shaders.

use std::{fs, path::PathBuf};

use ash::{prelude::VkResult, vk};

use crate::client::rendering::RenderResult;

pub struct ShaderModule {
    handle: vk::ShaderModule,
    device: ash::Device,
    path: PathBuf,
    bytecode: Option<Vec<u8>>,
}

impl ShaderModule {
    pub(super) fn new(device: ash::Device, create_info: &vk::ShaderModuleCreateInfo, path: PathBuf) -> VkResult<Self> {
        // SAFETY: The object is automatically dropped.
        Ok(
            Self {
                handle: unsafe { device.create_shader_module(create_info, None)? },
                device,
                path,
                bytecode: None,
            }
        )
    }

    pub fn read(&mut self) -> RenderResult<()> {
        self.bytecode = Some(fs::read(&self.path)?);
        Ok(())
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        // SAFETY: This is called upon dropping the shader module.
        unsafe {
            self.device.destroy_shader_module(self.handle, None);
        }
    }
}
