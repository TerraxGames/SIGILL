use ash::{ext, vk};
use sigill_derive::{Deref, DerefMut};

use super::RenderResult;

#[derive(Deref, DerefMut)]
/// An object with a custom destructor.
pub struct Object<T, D>(T, D, fn(&T, &D));

impl<T, D> Object<T, D> {
    pub fn new(object: T, data: D, destructor: fn(&T, &D)) -> Self {
        Self(object, data, destructor)
    }
}

impl<T, D> Drop for Object<T, D> {
    fn drop(&mut self) {
        (self.2)(&self.0, &self.1);
    }
}

// Some types for Object
pub type DebugUtilsMessenger = Object<vk::DebugUtilsMessengerEXT, ext::debug_utils::Instance>;

#[derive(Clone)]
pub struct Instance {
    inner: ash::Instance,
    extensions: Extensions,
}

impl Instance {
    pub fn new(entry: &ash::Entry, instance_info: &vk::InstanceCreateInfo) -> RenderResult<Self> {
        // SAFETY: The object is automatically dropped.
        let inner = unsafe { entry.create_instance(instance_info, None)?};
        Ok(Self {
            extensions: Extensions::new(entry, &inner),
            inner,
        })
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn create_debug_utils_messenger_ext(&self, create_info: &vk::DebugUtilsMessengerCreateInfoEXT) -> RenderResult<DebugUtilsMessenger> {
        // SAFETY: The object is automatically dropped.
        unsafe {
            Ok(
                Object::new(
                    self.extensions.debug_utils.create_debug_utils_messenger(create_info, None)?,
                    self.extensions.debug_utils.clone(),
                    |messenger, data| data.destroy_debug_utils_messenger(*messenger, None)
                )
            )
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // SAFETY: The object exists for the lifetime of this struct.
        unsafe { self.inner.destroy_instance(None); }
    }
}

#[derive(Clone)]
struct Extensions {
    pub debug_utils: ext::debug_utils::Instance,
}

impl Extensions {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        Self {
            debug_utils: ext::debug_utils::Instance::new(entry, instance),
        }
    }
}
