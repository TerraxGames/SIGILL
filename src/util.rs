use core::ops::{Deref, DerefMut};

/// A wrapper for Vulkan objects that require custom destructors.
pub struct VulkanObject<T>(Option<T>, fn(&T));

impl<T> VulkanObject<T> {
    pub fn new(object: T, destructor: fn(&T)) -> Self {
        Self(Some(object), destructor)
    }

    pub fn drop(&mut self) {
        self.1(self.deref());
    }
}

impl<T> Deref for VulkanObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for VulkanObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
