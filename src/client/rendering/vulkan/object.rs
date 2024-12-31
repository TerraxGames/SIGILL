//! A set of macros and abstractions for creating new Vulkan Objects.

use std::rc::Rc;

use ash::{ext, khr, vk};
use sigill_derive::{Deref, DerefMut};
use vk_mem::Alloc;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[macro_export]
/// A macro that reduces boilerplate by defining a struct with [`VulkanObject`].
/// 
/// Please note that any code inside `create` and `destroy` are marked as unsafe for convenience.
/// Additionally, the destroy method should *always* look like the code below. That is, the
/// parameter should never be anything but `self`. This is due to macro hygiene requirements.
/// ```no_run
/// fn destroy(self) {
/// 	// ...
/// }
/// ```
macro_rules! vulkan_object {
	(
		$name:ident<$create_info:ty, $object:ty, $data:ty>:
		fn create($create_info_name:ident, $data_name:ident) $create:block;
		fn destroy($self:ident) $destroy:block; // The $self is a hack that is required for macro hygiene. Thanks John Rust.
	) => {
		#[derive(sigill_derive::Deref, sigill_derive::DerefMut)]
		pub struct $name {
			object: $object,
			$data_name: $data,
		}

		impl $crate::client::rendering::vulkan::object::VulkanObject<$create_info, $object, $data> for $name {
			unsafe fn new($create_info_name: &$create_info, $data_name: $data) -> ash::prelude::VkResult<Self> {
				Ok(
					Self {
						object: unsafe { Self::create($create_info_name, &$data_name)? },
						$data_name,
					}
				)
			}

			unsafe fn create($create_info_name: &$create_info, $data_name: &$data) -> ash::prelude::VkResult<$object> {
				unsafe { $create }
			}

			unsafe fn destroy(&mut $self) {
				unsafe { $destroy }
			}
		}

		impl core::ops::Drop for $name {
			fn drop(&mut self) {
				unsafe { self.destroy(); }
			}
		}
	};
}

/// A weak reference to a Vulkan object.
/// Typically, this type is used when the lifetime of another type is tied to the underlying
/// Vulkan object, so using [`VulkanObject`] is unnecessary or unsound. Thus, [`WeakRef`]
/// fills that niche.
/// 
/// # Implementation
/// The underlying type is the inner object of a [`VulkanObject`].
/// 
/// # Safety
/// You must ensure that the underlying Vulkan object is still
/// initialized in Vulkan before using it.
#[derive(Deref, DerefMut)]
pub struct WeakRef<T>(T);

/// An object with a custom destructor.
/// This struct is used for Vulkan objects that require special allocation handling.
/// # Necessity
/// All Vulkan objects constructed via `vkCreateXXXX` functions are required to be destroyed with their accompanying `vkDestroyXXXX` functions.
/// This type serves as a utility for automatically destroying each Vulkan object upon being dropped.
/// 
/// See [`vulkan_object`].
pub trait VulkanObject<C, T, D>: Sized {
	/// # Safety
	/// Safety is assumed when the data has not yet been uninitialized.
	unsafe fn new(create_info: &C, data: D) -> ash::prelude::VkResult<Self>;

	/// # Safety
	/// Safety is assumed when the data has not yet been uninitialized.
	unsafe fn create(create_info: &C, data: &D) -> ash::prelude::VkResult<T>;

	/// # Safety
	/// Safety is assumed when the data has not yet been uninitialized.
	unsafe fn destroy(&mut self);
}

/// A "functional trait" for [`VulkanObject`] implementors that returns a weak reference
/// to the underlying Vulkan object.
pub trait WeakObject<T> {
	/// # Safety
	/// Using [`WeakRef`] has some extra safety concerns.
	unsafe fn object(&self) -> WeakRef<T>;

	/// # Safety
	/// See [`WeakRef`].
	unsafe fn from_object(object: T) -> Self;
}

vulkan_object!{
	DebugUtilsMessenger<vk::DebugUtilsMessengerCreateInfoEXT<'_>, vk::DebugUtilsMessengerEXT, ext::debug_utils::Instance>:
	fn create(create_info, data) {
		data.create_debug_utils_messenger(create_info, None)
	};
	fn destroy(self) {
		self.data.destroy_debug_utils_messenger(self.object, None)
	};
}

vulkan_object!{
	Surface<(RawDisplayHandle, RawWindowHandle), vk::SurfaceKHR, (khr::surface::Instance, ash::Entry, ash::Instance)>:
	fn create(handles, data) {
		ash_window::create_surface(&data.1, &data.2, handles.0, handles.1, None)
	};
	fn destroy(self) {
		self.data.0.destroy_surface(self.object, None);
	};
}

vulkan_object!{
	ImageView<vk::ImageViewCreateInfo<'_>, vk::ImageView, ash::Device>:
	fn create(create_info, device) {
		device.create_image_view(create_info, None)
	};
	fn destroy(self) {
		self.device.destroy_image_view(self.object, None);
	};
}

pub struct Image {
	object: vk::Image,
	allocation: Option<vk_mem::Allocation>,
	allocator: Option<Rc<vk_mem::Allocator>>,
}

impl VulkanObject<
	(vk::ImageCreateInfo<'_>, vk_mem::AllocationCreateInfo),
	(vk::Image, vk_mem::Allocation),
	Rc<vk_mem::Allocator>,
> for Image {
	unsafe fn new(
		create_info: &(vk::ImageCreateInfo<'_>, vk_mem::AllocationCreateInfo),
		allocator: Rc<vk_mem::Allocator>,
	) -> ash::prelude::VkResult<Self> {
		let (object, allocation) = unsafe { Self::create(create_info, &allocator)? };
		Ok(
			Self {
				object,
				allocation: Some(allocation),
				allocator: Some(allocator),
			}
		)
	}

	unsafe fn create(
		create_info: &(vk::ImageCreateInfo<'_>, vk_mem::AllocationCreateInfo),
		allocator: &Rc<vk_mem::Allocator>
	) -> ash::prelude::VkResult<(vk::Image, vk_mem::Allocation)> {
		let (create_info, allocation_create_info) = create_info;
		unsafe { allocator.create_image(create_info, allocation_create_info) }
	}

	unsafe fn destroy(&mut self) {
		if let Some(allocator) = &self.allocator {
			if let Some(allocation) = &mut self.allocation {
				unsafe { allocator.destroy_image(self.object, allocation); }
			}
		}
	}
}

impl WeakObject<vk::Image> for Image {
	unsafe fn object(&self) -> WeakRef<vk::Image> {
		WeakRef(self.object.clone())
	}
	
	unsafe fn from_object(object: vk::Image) -> Self {
		Self {
			object,
			allocation: None,
			allocator: None,
		}
	}
}
