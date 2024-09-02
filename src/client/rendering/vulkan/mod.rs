//! # Vulkan Safety Abstractions
//! This module provides safe abstractions for Vulkan objects.
//!
//! See [`VulkanObject`] and [`Instance`].

use std::{any::Any, collections::HashMap, mem::ManuallyDrop, ptr::drop_in_place, u32};

use ash::{ext, khr, prelude::VkResult, vk};
use sigill_derive::{Deref, DerefMut};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::RenderResult;

pub use swapchain::*;

pub mod swapchain;

pub type QueueFamilyIndex = u32;
pub type QueueIndex = u32;

/// An object with a custom destructor.
/// This struct is used for Vulkan objects that require special allocation handling.
/// # Necessity
/// All Vulkan objects constructed via `vkCreateXXXX` functions are required to be destroyed with their accompanying `vkDestroyXXXX` functions.
/// This type serves as a utility for automatically destroying each Vulkan object upon being dropped.
/// 
/// See [`VulkanObjectType`].
#[derive(Deref, DerefMut)]
pub struct VulkanObject<T, D>(T, D, fn(&T, &D));

impl<T, D> VulkanObject<T, D> {
    pub fn new(object: T, data: D, destructor: fn(&T, &D)) -> Self {
        Self(object, data, destructor)
    }
}

impl<T, D> Drop for VulkanObject<T, D> {
    fn drop(&mut self) {
        (self.2)(&self.0, &self.1);
    }
}

// Some types for Object
pub type DebugUtilsMessenger = VulkanObject<vk::DebugUtilsMessengerEXT, ext::debug_utils::Instance>;
pub type Surface = VulkanObject<vk::SurfaceKHR, khr::surface::Instance>;

/// A type of Vulkan object that is automatically dropped in order of dependency.
/// # Safety
/// All object types must declared be below their dependents since objects are dropped in the order of their discriminant.
#[repr(u32)]
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VulkanObjectType {
    Swapchain,

    Surface,

    Device,

    // Drop the debug messenger last just in case we mess up Vulkan object destruction.
    DebugUtilsMessenger,
}

/// The struct that owns all Vulkan objects.
pub struct Instance {
    /// An abstraction for handling inherited Vulkan objects.
    objects: ManuallyDrop<HashMap<VulkanObjectType, Box<dyn Any>>>,
    extensions: Extensions,
    inner: ash::Instance,
    entry: ash::Entry,
}

impl Instance {
    pub fn new(entry: ash::Entry, instance_info: &vk::InstanceCreateInfo) -> RenderResult<Self> {
        // SAFETY: The object is automatically dropped.
        let inner = unsafe { entry.create_instance(instance_info, None)?};
        Ok(Self {
            objects: ManuallyDrop::new(HashMap::new()),
            extensions: Extensions::new(&entry, &inner),
            inner,
            entry,
        })
    }

    // Vulkan Object Management

    #[inline]
    pub fn debug_utils_messenger(&self) -> &DebugUtilsMessenger {
        self.get_object(VulkanObjectType::DebugUtilsMessenger).expect("debug_utils_messenger must be initialized before being accessed")
    }

    #[inline]
    pub fn swapchain(&self) -> &Swapchain {
        self.get_object(VulkanObjectType::Swapchain).expect("swapchain must be initialized before being accessed")
    }

    #[inline]
    pub fn surface(&self) -> &Surface {
        self.get_object(VulkanObjectType::Surface).expect("surface must be initialized before being accessed")
    }

    #[inline]
    pub fn device(&self) -> &Device {
        self.get_object(VulkanObjectType::Device).expect("device must be initialized before being accessed")
    }

    #[inline]
    pub fn get_object<T: Any>(&self, object_type: VulkanObjectType) -> Option<&T> {
        self.objects.get(&object_type)?.downcast_ref()
    }

    pub fn set_object<T: Any>(&mut self, object_type: VulkanObjectType, object: T) {
        self.objects.insert(object_type, Box::new(object));
    }

    #[inline]
    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    // Extensions

    #[inline]
    pub fn get_physical_device_surface_support(&self, physical_device: vk::PhysicalDevice, queue_family_index: QueueFamilyIndex, surface: &Surface) -> VkResult<bool> {
        // SAFETY: The object needs no additional allocation.
        unsafe { self.extensions.surface.get_physical_device_surface_support(physical_device, queue_family_index, surface.0) }
    }

    #[inline]
    pub fn get_physical_device_surface_capabilities(&self, physical_device: vk::PhysicalDevice, surface: &Surface) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.extensions.surface.get_physical_device_surface_capabilities(physical_device, surface.0) }
    }

    #[inline]
    pub fn get_physical_device_surface_formats(&self, physical_device: vk::PhysicalDevice, surface: &Surface) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.extensions.surface.get_physical_device_surface_formats(physical_device, surface.0) }
    }

    #[inline]
    pub fn get_physical_device_surface_present_modes(&self, physical_device: vk::PhysicalDevice, surface: &Surface) -> VkResult<Vec<vk::PresentModeKHR>> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.extensions.surface.get_physical_device_surface_present_modes(physical_device, surface.0) }
    }

    // Vulkan Object Creation for Extensions

    #[inline]
    pub fn create_debug_utils_messenger_ext(&mut self, create_info: &vk::DebugUtilsMessengerCreateInfoEXT) -> VkResult<&DebugUtilsMessenger> {
        // SAFETY: The object is automatically dropped.
        self.set_object(
            VulkanObjectType::DebugUtilsMessenger,
            unsafe {
                VulkanObject::new(
                    self.extensions.debug_utils.create_debug_utils_messenger(create_info, None)?,
                    self.extensions.debug_utils.clone(),
                    |messenger, data| data.destroy_debug_utils_messenger(*messenger, None)
                )
            },
        );
        Ok(self.debug_utils_messenger())
    }

    /// This method creates a singleton swapchain.
    #[inline]
    pub fn create_swapchain(&mut self, create_info: &vk::SwapchainCreateInfoKHR) -> VkResult<&Swapchain> {
        let swapchain_device = khr::swapchain::Device::new(&self.inner, &self.device().inner);
        // SAFETY: The object is automatically dropped.
        self.set_object(
            VulkanObjectType::Swapchain,
            unsafe {
                let handle = swapchain_device.create_swapchain(create_info, None)?;
                let images = swapchain_device.get_swapchain_images(handle)?;
                Swapchain::new(
                    handle,
                    swapchain_device,
                    images,
                    create_info.image_format,
                    create_info.image_extent,
                )
            }
        );
        Ok(self.swapchain())
    }

    // Vulkan Object Creation
    
    #[inline]
    pub fn create_surface(&mut self, display_handle: RawDisplayHandle, window_handle: RawWindowHandle) -> VkResult<&Surface> {
        // SAFETY: The object is automatically dropped.
        self.set_object(
            VulkanObjectType::Surface, 
            unsafe {
                VulkanObject::new(
                    ash_window::create_surface(self.entry(), &self.inner, display_handle, window_handle, None)?,
                    khr::surface::Instance::new(self.entry(), &self.inner),
                    |surface, instance| instance.destroy_surface(*surface, None),
                )
            },
        );
        Ok(self.surface())
    }

    #[inline]
    pub fn create_device(&mut self, physical_device: vk::PhysicalDevice, create_info: &vk::DeviceCreateInfo) -> VkResult<&Device> {
        // SAFETY: The object is automatically dropped.
        self.set_object(
            VulkanObjectType::Device,
            unsafe {
                Device {
                    inner: self.inner.create_device(physical_device, create_info, None)?,
                }
            },
        );
        Ok(self.device())
    }

    // Inner Instance Methods

    #[inline]
    pub fn enumerate_physical_devices(&self) -> VkResult<Vec<vk::PhysicalDevice>> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.inner.enumerate_physical_devices() }
    }

    #[inline]
    pub fn get_physical_device_properties(&self, physical_device: vk::PhysicalDevice) -> vk::PhysicalDeviceProperties {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.inner.get_physical_device_properties(physical_device) }
    }

    #[inline]
    pub fn get_physical_device_features(&self, physical_device: vk::PhysicalDevice) -> vk::PhysicalDeviceFeatures {
        // SAFETY: The object needs to additional allocation function.
        unsafe { self.inner.get_physical_device_features(physical_device) }
    }

    #[inline]
    pub fn get_physical_device_queue_family_properties(&self, physical_device: vk::PhysicalDevice) -> Vec<vk::QueueFamilyProperties> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.inner.get_physical_device_queue_family_properties(physical_device) }
    }

    #[inline]
    pub fn enumerate_device_extension_properties(&self, physical_device: vk::PhysicalDevice) -> VkResult<Vec<vk::ExtensionProperties>> {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.inner.enumerate_device_extension_properties(physical_device) }
    }

    // Helper Methods
    
    /// # Parameter Guarantee
    /// The `queue_flags` parameter is assumed to contain only one flag per element.
    /// This is so that each flag can be indexed in the resulting [`HashMap`] via a single [`vk::QueueFlags`].
    /// However, if you require multiple types of queues per queue family, you may add multiple flags to an element.
    pub fn get_queue_family_map(&self, physical_device: vk::PhysicalDevice, queue_flags: &[vk::QueueFlags]) -> QueueFamilyMap {
        let mut map = HashMap::new();
        let queue_families = self.get_physical_device_queue_family_properties(physical_device);
        for (queue_family_index, queue_family) in queue_families.iter().enumerate() {
            let mut queue_index = 0; // the index within the queue family
            for queue_flag in queue_flags.iter() {
                if queue_family.queue_flags.contains(*queue_flag) && !map.contains_key(queue_flag) {
                    map.insert(*queue_flag, (queue_family_index as u32, queue_index as u32));
                    queue_index += 1; // increment the queue index once we've added one to the queue family
                }
            }
        }
        QueueFamilyMap {
            inner: map,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // Sort objects to drop by their discriminant (i.e. their drop order).
        let mut sorted_objects = Vec::new();
        sorted_objects.extend(self.objects.iter_mut());
        sorted_objects.sort_by(|x, y| x.0.cmp(y.0));
        for (_, object) in sorted_objects {
            // SAFETY: The value is dropped during this struct's destructor, and it is not accessed again.
            unsafe { drop_in_place(object.as_mut()); }
        }

        // SAFETY: The object exists for the lifetime of this struct.
        unsafe { self.inner.destroy_instance(None); }
    }
}

#[derive(Clone)]
struct Extensions {
    pub debug_utils: ext::debug_utils::Instance,
    pub surface: khr::surface::Instance,
}

impl Extensions {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        Self {
            debug_utils: ext::debug_utils::Instance::new(entry, instance),
            surface: khr::surface::Instance::new(entry, instance),
        }
    }
}

#[repr(transparent)]
pub struct Device {
    inner: ash::Device,
}

impl Device {
    #[inline]
    pub fn get_device_queue(&self, queue_family_index: QueueFamilyIndex, queue_index: QueueIndex) -> vk::Queue {
        // SAFETY: The object needs no additional allocation function.
        unsafe { self.inner.get_device_queue(queue_family_index, queue_index) }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // SAFETY: The object exists for the lifetime of this struct.
        unsafe { self.inner.destroy_device(None); }
    }
}

#[repr(transparent)]
pub struct QueueFamilyMap {
    inner: HashMap<vk::QueueFlags, (QueueFamilyIndex, QueueIndex)>,
}

impl QueueFamilyMap {
    pub fn get_queue_info(&self, queue_flags: vk::QueueFlags) -> Option<&(QueueFamilyIndex, QueueIndex)> {
        self.inner.get(&queue_flags)
    }

    pub fn inner(&self) -> &HashMap<vk::QueueFlags, (QueueFamilyIndex, QueueIndex)> {
        &self.inner
    }
}

impl std::fmt::Debug for QueueFamilyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(&self.inner).finish()
    }
}