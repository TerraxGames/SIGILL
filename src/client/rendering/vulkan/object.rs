//! A set of macros and abstractions for creating new Vulkan Objects.

use ash::{ext, vk};

#[macro_export]
macro_rules! vulkan_object {
	(
		$name:ident<$create_info:ty, $object:ty, $data:ty>:
		fn create($create_info_name:ident, $data_name:ident) $create:block;
		fn destroy($self:ident) $destroy:block;
	) => {
		#[derive(sigill_derive::Deref, sigill_derive::DerefMut)]
		pub struct $name {
			object: $object,
			data: $data,
		}

		impl $crate::client::rendering::vulkan::object::VulkanObject<$create_info, $object, $data> for $name {
			unsafe fn new(create_info: &$create_info, data: $data) -> ash::prelude::VkResult<Self> {
				Ok(
					Self {
						object: unsafe { Self::create(create_info, &data)? },
						data,
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

vulkan_object!{
	DebugUtilsMessenger<vk::DebugUtilsMessengerCreateInfoEXT<'_>, vk::DebugUtilsMessengerEXT, ext::debug_utils::Instance>:
	fn create(create_info, data) {
		data.create_debug_utils_messenger(create_info, None)
	};
	fn destroy(self) {
		self.data.destroy_debug_utils_messenger(self.object, None)
	};
}
