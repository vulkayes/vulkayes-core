//! An instance represents an instance of Vulkan application.

use std::{
	ffi::{CStr, CString},
	fmt::{Debug, Error, Formatter},
	ops::Deref,
	os::raw::c_char
};

use ash::{extensions::ext::DebugUtils, vk};

use crate::{
	entry::Entry,
	memory::host::HostMemoryAllocator,
	physical_device::PhysicalDevice,
	prelude::Vrc,
	util::fmt::VkVersion
};

pub mod debug;
pub mod error;
#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, Copy, Default)]
pub struct ApplicationInfo<'a> {
	pub application_name: &'a str,
	pub engine_name: &'a str,
	pub application_version: VkVersion,
	pub engine_version: VkVersion,
	pub api_version: VkVersion
}

struct InstanceDebug {
	loader: DebugUtils,
	callback: vk::DebugUtilsMessengerEXT,
	host_memory_allocator: HostMemoryAllocator
}
impl Debug for InstanceDebug {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("InstanceDebug")
			.field("loader", &"<ash::_::DebugReport>")
			.field("callback", &self.callback)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
pub struct Instance {
	entry: Entry,
	instance: ash::Instance,
	// For the HasHandle trait
	instance_handle: vk::Instance,
	host_memory_allocator: HostMemoryAllocator,

	debug: Option<InstanceDebug>
}
impl Instance {
	/// Creates a new instance from an existing entry.
	pub fn new<'a>(
		entry: Entry,
		application_info: ApplicationInfo,
		layers: impl IntoIterator<Item = &'a CStr> + std::fmt::Debug,
		extensions: impl IntoIterator<Item = &'a CStr> + std::fmt::Debug,
		host_memory_allocator: HostMemoryAllocator,
		debug_callback: debug::DebugCallback
	) -> Result<Vrc<Self>, error::InstanceError> {
		log::info!("Vulkan instance version {}", entry.instance_version());

		let application_name_c = CString::new(application_info.application_name)?;
		let engine_name_c = CString::new(application_info.engine_name)?;

		let app_info = vk::ApplicationInfo::builder()
			.application_name(application_name_c.as_ref())
			.engine_name(engine_name_c.as_ref())
			.application_version(application_info.application_version.0)
			.engine_version(application_info.engine_version.0)
			.api_version(application_info.api_version.0);

		log::debug!(
			"Instance create info {:#?} {:#?} {:#?}",
			application_info,
			layers,
			extensions
		);

		let ptr_layers: Vec<*const c_char> = layers.into_iter().map(CStr::as_ptr).collect();
		let ptr_extensions: Vec<*const c_char> = extensions.into_iter().map(CStr::as_ptr).collect();
		let create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_layer_names(ptr_layers.as_slice())
			.enabled_extension_names(ptr_extensions.as_slice());

		unsafe {
			Instance::from_create_info(entry, create_info, host_memory_allocator, debug_callback)
		}
	}

	/// Creates a new `Instance` from existing `InstanceCreateInfo`.
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkInstanceCreateInfo.html>.
	pub unsafe fn from_create_info(
		entry: Entry,
		create_info: impl Deref<Target = ash::vk::InstanceCreateInfo>,
		host_memory_allocator: HostMemoryAllocator,
		debug_callback: debug::DebugCallback
	) -> Result<Vrc<Self>, error::InstanceError> {
		log_trace_common!(
			"Creating instance:",
			entry,
			create_info.deref(),
			host_memory_allocator,
			debug_callback
		);
		let instance = entry.create_instance(&create_info, host_memory_allocator.as_ref())?;

		// TODO: debug messenger, validation features, validation flags?

		let debug = match debug_callback.into() {
			None => None,
			Some(ref create_info) => {
				let loader = DebugUtils::new(entry.deref(), &instance);
				let callback = loader.create_debug_utils_messenger(create_info, None)?;

				Some(InstanceDebug {
					loader,
					callback,
					host_memory_allocator: HostMemoryAllocator::Unspecified() /* TODO: Allow callbacks */
				})
			}
		};

		Ok(Vrc::new(Instance {
			entry,
			instance_handle: instance.handle(),
			instance,
			host_memory_allocator,
			debug
		}))
	}

	pub const fn entry(&self) -> &Entry {
		&self.entry
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumeratePhysicalDevices.html>.
	pub fn physical_devices(
		self: &Vrc<Self>
	) -> Result<impl ExactSizeIterator<Item = PhysicalDevice>, error::PhysicalDeviceEnumerationError>
	{
		let elf = self.clone();
		let enumerator = unsafe {
			self.enumerate_physical_devices()?
				.into_iter()
				.map(move |physical_device| {
					PhysicalDevice::from_existing(elf.clone(), physical_device)
				})
		};

		Ok(enumerator)
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Instance>, Borrow, Eq, Hash, Ord for Instance {
		target = { instance_handle }
	}
}
impl Deref for Instance {
	type Target = ash::Instance;

	fn deref(&self) -> &Self::Target {
		&self.instance
	}
}
impl Drop for Instance {
	fn drop(&mut self) {
		log_trace_common!(info; "Dropping", self);

		unsafe {
			if let Some(debug) = self.debug.as_mut() {
				debug.loader.destroy_debug_utils_messenger(
					debug.callback,
					debug.host_memory_allocator.as_ref()
				);
			}
			self.instance
				.destroy_instance(self.host_memory_allocator.as_ref());
		}
	}
}
impl Debug for Instance {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("Instance")
			.field("entry", &self.entry)
			.field(
				"instance",
				&crate::util::fmt::format_handle(self.instance.handle())
			)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.field("debug", &self.debug)
			.finish()
	}
}
