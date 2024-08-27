use ash::vk;

use super::RenderResult;

#[repr(transparent)]
#[derive(Clone)]
pub struct Instance(ash::Instance);

impl Instance {
    pub fn new(entry: &ash::Entry, instance_info: &vk::InstanceCreateInfo) -> RenderResult<Self> {
        // SAFETY: The object is automatically dropped.
        Ok(Self(unsafe { entry.create_instance(instance_info, None)? }))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // SAFETY: The object exists for the lifetime of this struct.
        unsafe { self.0.destroy_instance(None); }
    }
}
