use std::convert::TryInto;
use std::ffi::{CString, NulError};
use std::os::raw::c_char;

use ash::extensions::ext::DebugReport;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::AllocationCallbacks;

use crate::entry::Entry;
use crate::memory::host::HostMemoryAllocator;

pub mod debug;
pub mod error;

#[derive(Debug, Clone, Copy)]
pub struct ApplicationInfo<'a> {
	pub application_name: &'a str,
	pub engine_name: &'a str,
	pub application_version: u32,
	pub engine_version: u32,
	pub api_version: u32
}
impl<'a> TryInto<ash::vk::ApplicationInfo> for ApplicationInfo<'a> {
	type Error = NulError;

	fn try_into(self) -> Result<ash::vk::ApplicationInfo, Self::Error> {
		Ok(
			ash::vk::ApplicationInfo::builder()
				.application_name(
					CString::new(self.application_name)?.as_ref()
				)
				.engine_name(
					CString::new(self.engine_name)?.as_ref()
				)
				.application_version(self.application_version)
				.engine_version(self.engine_version)
				.api_version(self.api_version)
				.build()
		)
	}
}

pub struct Instance {
	entry: Entry,
	instance: ash::Instance,
	allocation_callbacks: Option<AllocationCallbacks>,

	debug: Option<InstanceDebug>,
}
impl Instance {
	/// Creates a new instance from an existing entry.
	pub fn new<'a>(
		entry: Entry,
		application_info: ApplicationInfo,
		layers: impl IntoIterator<Item = &'a str>,
		extensions: impl IntoIterator<Item = &'a str>,
		host_memory_allocator: HostMemoryAllocator,
		debug_callback: debug::DebugCallback,
	) -> Result<Self, error::InstanceError> {
		let app_info = application_info.try_into()?;

		let cstr_layers = layers.into_iter().map(CString::new).collect::<Result<Vec<_>, _>>()?;
		let ptr_layers: Vec<*const c_char> = cstr_layers.iter().map(|cstr| cstr.as_ptr()).collect();

		let cstr_extensions = extensions.into_iter().map(CString::new).collect::<Result<Vec<_>, _>>()?;
		let ptr_extensions: Vec<*const c_char> = cstr_extensions.iter().map(|cstr| cstr.as_ptr()).collect();

		let create_info = ash::vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_layer_names(ptr_layers.as_slice())
			.enabled_extension_names(ptr_extensions.as_slice())
			.build();

		let allocation_callbacks: Option<AllocationCallbacks> = host_memory_allocator.into();

		let instance = unsafe {
			log::debug!("Creating instance with {:?} layers: {:?} extensions: {:?} allocation_callbacks: {:?}", application_info, cstr_layers, cstr_extensions, allocation_callbacks);

			entry.as_ref().create_instance(
				&create_info,
				allocation_callbacks.as_ref(),
			)?
		};

		let debug = match debug_callback.into() {
			None => None,
			Some(ref create_info) => {
				let loader = DebugReport::new(entry.as_ref(), &instance);
				let callback = unsafe {
					loader.create_debug_report_callback(create_info, None)?
				};

				Some(
					InstanceDebug {
						loader,
						callback,
						allocation_callbacks: None, // TODO: Allow callbacks
					}
				)
			}
		};

		Ok(
			Instance {
				entry,
				instance,
				allocation_callbacks,

				debug,
			}
		)
	}
}
impl Drop for Instance {
	fn drop(&mut self) {
		unsafe {
			self.instance.destroy_instance(self.allocation_callbacks.as_ref());
		}
	}
}

struct InstanceDebug {
	loader: DebugReport,
	callback: ash::vk::DebugReportCallbackEXT,
	allocation_callbacks: Option<AllocationCallbacks>,
}
impl Drop for InstanceDebug {
	fn drop(&mut self) {
		unsafe {
			self.loader.destroy_debug_report_callback(self.callback, self.allocation_callbacks.as_ref());
		}
	}
}