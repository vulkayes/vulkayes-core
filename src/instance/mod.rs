//! An instance represents an instance of Vulkan application.

use std::{
	ffi::CString,
	fmt::{Debug, Error, Formatter},
	ops::Deref,
	os::raw::c_char
};

use ash::{
	extensions::ext::DebugReport,
	version::{EntryV1_0, InstanceV1_0},
	vk::AllocationCallbacks
};

use crate::{
	entry::Entry,
	memory::host::HostMemoryAllocator,
	physical_device::PhysicalDevice,
	util::fmt::VkVersion,
	Vrc
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
	loader: DebugReport,
	callback: ash::vk::DebugReportCallbackEXT,
	allocation_callbacks: Option<AllocationCallbacks>
}
impl Debug for InstanceDebug {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("InstanceDebug")
			.field("loader", &"<ash::_::DebugReport>")
			.field("callback", &self.callback)
			.field("allocation_callbacks", &self.allocation_callbacks)
			.finish()
	}
}
pub struct Instance {
	entry: Entry,
	instance: ash::Instance,
	allocation_callbacks: Option<AllocationCallbacks>,

	debug: Option<InstanceDebug>
}
impl Instance {
	/// Creates a new instance from an existing entry.
	pub fn new<'a>(
		entry: Entry,
		application_info: ApplicationInfo,
		layers: impl IntoIterator<Item = &'a str>,
		extensions: impl IntoIterator<Item = &'a str>,
		host_memory_allocator: HostMemoryAllocator,
		debug_callback: debug::DebugCallback
	) -> Result<Vrc<Self>, error::InstanceError> {
		let application_name_c = CString::new(application_info.application_name)?;
		let engine_name_c = CString::new(application_info.engine_name)?;

		let app_info = ash::vk::ApplicationInfo::builder()
			.application_name(application_name_c.as_ref())
			.engine_name(engine_name_c.as_ref())
			.application_version(application_info.application_version.0)
			.engine_version(application_info.engine_version.0)
			.api_version(application_info.api_version.0);

		let cstr_layers = layers
			.into_iter()
			.map(CString::new)
			.collect::<Result<Vec<_>, _>>()?;
		let ptr_layers: Vec<*const c_char> = cstr_layers.iter().map(|cstr| cstr.as_ptr()).collect();

		let cstr_extensions = extensions
			.into_iter()
			.map(CString::new)
			.collect::<Result<Vec<_>, _>>()?;
		let ptr_extensions: Vec<*const c_char> =
			cstr_extensions.iter().map(|cstr| cstr.as_ptr()).collect();

		log::debug!(
			"Instance create info {:#?} {:#?} {:#?}",
			application_info,
			cstr_layers,
			cstr_extensions
		);
		let create_info = ash::vk::InstanceCreateInfo::builder()
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
		let allocation_callbacks: Option<AllocationCallbacks> = host_memory_allocator.into();

		log::debug!(
			"Creating instance with {:#?} {:#?}",
			create_info.deref(),
			allocation_callbacks
		);
		let instance = entry.create_instance(&create_info, allocation_callbacks.as_ref())?;

		// TODO: debug messenger, validation features, validation flags?

		let debug = match debug_callback.into() {
			None => None,
			Some(ref create_info) => {
				let loader = DebugReport::new(entry.deref(), &instance);
				let callback = loader.create_debug_report_callback(create_info, None)?;

				Some(InstanceDebug {
					loader,
					callback,
					allocation_callbacks: None // TODO: Allow callbacks
				})
			}
		};

		Ok(Vrc::new(Instance {
			entry,
			instance,
			allocation_callbacks,
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
				.map(move |physical_device| PhysicalDevice::new(elf.clone(), physical_device))
		};

		Ok(enumerator)
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
		unsafe {
			if let Some(debug) = self.debug.as_mut() {
				debug.loader.destroy_debug_report_callback(
					debug.callback,
					debug.allocation_callbacks.as_ref()
				);
			}
			self.instance
				.destroy_instance(self.allocation_callbacks.as_ref());
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
			.field("allocation_callbacks", &self.allocation_callbacks)
			.field("debug", &self.debug)
			.finish()
	}
}
