//! A device represents an instance of connection to a physical device.

use std::{ffi::CStr, fmt::Debug, ops::Deref, os::raw::c_char};

use ash::{
	version::{DeviceV1_0, InstanceV1_0},
	vk::{self, DeviceCreateInfo, DeviceQueueCreateInfo}
};

use crate::{
	instance::Instance,
	memory::host::HostMemoryAllocator,
	physical_device::PhysicalDevice,
	prelude::Vrc,
	queue::Queue
};

pub mod error;

#[derive(Debug, Clone, Copy)]
pub struct QueueCreateInfo<P: AsRef<[f32]>> {
	pub queue_family_index: u32,
	pub queue_priorities: P
}

/// Return type of `Device` constructors.
#[derive(Debug)]
pub struct DeviceData {
	pub device: Vrc<Device>,
	pub queues: Vec<Vrc<Queue>>
}

pub struct Device {
	device: ash::Device,
	device_handle: vk::Device,

	physical_device: PhysicalDevice,

	host_memory_allocator: HostMemoryAllocator
}
impl Device {
	pub fn new<'a, P: AsRef<[f32]> + Debug>(
		physical_device: PhysicalDevice,
		queues: impl AsRef<[QueueCreateInfo<P>]>,
		layers: impl IntoIterator<Item = &'a CStr> + std::fmt::Debug,
		extensions: impl IntoIterator<Item = &'a CStr> + std::fmt::Debug,
		features: vk::PhysicalDeviceFeatures,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<DeviceData, error::DeviceError> {
		let queues = queues.as_ref();

		#[cfg(feature = "runtime_implicit_validations")]
		{
			if queues.len() == 0 {
				return Err(error::DeviceError::QueuesEmpty)
			}
			if queues
				.iter()
				.any(|c| c.queue_priorities.as_ref().len() == 0)
			{
				return Err(error::DeviceError::QueuePrioritiesEmpty)
			}
		}

		// create info pointers are valid because they are kept alive by queues argument
		let queue_create_infos: Vec<_> = queues
			.iter()
			.map(|q| {
				DeviceQueueCreateInfo::builder()
					.queue_family_index(q.queue_family_index)
					.queue_priorities(q.queue_priorities.as_ref())
					.build()
			})
			.collect();

		log::debug!(
			"Device create info {:#?} {:#?} {:#?} {:#?}",
			queues,
			layers,
			extensions,
			features
		);

		let ptr_layers: Vec<*const c_char> = layers.into_iter().map(CStr::as_ptr).collect();
		let ptr_extensions: Vec<*const c_char> = extensions.into_iter().map(CStr::as_ptr).collect();
		let create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(&queue_create_infos)
			.enabled_layer_names(ptr_layers.as_slice())
			.enabled_extension_names(ptr_extensions.as_slice())
			.enabled_features(&features);

		unsafe { Device::from_create_info(physical_device, create_info, host_memory_allocator) }
	}

	/// Creates a new `Device` from existing `DeviceCreateInfo`
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDeviceCreateInfo.html>.
	pub unsafe fn from_create_info(
		physical_device: PhysicalDevice,
		create_info: impl Deref<Target = DeviceCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<DeviceData, error::DeviceError> {
		log_trace_common!(
			"Creating device:",
			physical_device,
			create_info.deref(),
			host_memory_allocator
		);
		let device = physical_device.instance().create_device(
			*physical_device,
			&create_info,
			host_memory_allocator.as_ref()
		)?;

		let device = Vrc::new(Device {
			device_handle: device.handle(),
			device,
			physical_device,
			host_memory_allocator
		});
		let queues = device.get_created_queues(create_info);

		Ok(DeviceData { device, queues })
	}

	unsafe fn get_created_queues(
		self: &Vrc<Self>,
		create_info: impl Deref<Target = DeviceCreateInfo>
	) -> Vec<Vrc<Queue>> {
		let num = create_info.queue_create_info_count as usize;
		let mut result = Vec::with_capacity(num);

		for family in 0 .. num as isize {
			let info = &*create_info.p_queue_create_infos.offset(family);

			for index in 0 .. info.queue_count {
				result.push(Queue::from_device(
					self.clone(),
					info.flags,
					info.queue_family_index,
					index
				));
			}
		}

		result
	}

	pub fn wait_idle(&self) -> Result<(), error::DeviceWaitError> {
		unsafe { self.device.device_wait_idle().map_err(Into::into) }
	}

	pub const fn physical_device(&self) -> &PhysicalDevice {
		&self.physical_device
	}

	pub const fn instance(&self) -> &Vrc<Instance> {
		self.physical_device.instance()
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Device>, Borrow, Eq, Hash, Ord for Device {
		target = { device_handle }
	}
}
impl Deref for Device {
	type Target = ash::Device;

	fn deref(&self) -> &Self::Target {
		&self.device
	}
}
impl Drop for Device {
	fn drop(&mut self) {
		log_trace_common!(info; "Dropping", self);

		let _ = self.wait_idle();
		unsafe {
			self.device
				.destroy_device(self.host_memory_allocator.as_ref());
		}
	}
}
impl Debug for Device {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("Device")
			.field(
				"device",
				&crate::util::fmt::format_handle(self.device.handle())
			)
			.field("physical_device", &self.physical_device)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
