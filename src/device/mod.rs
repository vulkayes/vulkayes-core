use std::{ffi::CString, fmt::Debug, os::raw::c_char};
use std::fmt::Formatter;
use std::ops::Deref;

use ash::{
	version::{DeviceV1_0, InstanceV1_0},
	vk::{AllocationCallbacks, DeviceCreateInfo, DeviceQueueCreateFlags, DeviceQueueCreateInfo}
};

use crate::{
	instance::Instance,
	memory::host::HostMemoryAllocator,
	physical_device::PhysicalDevice,
	Vrc
};
use crate::queue::Queue;

pub mod error;
#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, Copy)]
pub struct QueueCreateInfo<P: AsRef<[f32]>> {
	pub queue_family_index: u32,
	pub flags: DeviceQueueCreateFlags,
	pub queue_priorities: P
}

pub struct Device {
	#[allow(dead_code)] // Need to keep the instance alive
	instance: Vrc<Instance>,
	device: ash::Device,

	physical_device: PhysicalDevice,

	allocation_callbacks: Option<AllocationCallbacks>
}
impl Device {
	pub fn new<'a, P: AsRef<[f32]> + Debug>(
		instance: Vrc<Instance>, queues: impl AsRef<[QueueCreateInfo<P>]>,
		layers: impl IntoIterator<Item = &'a str>, extensions: impl IntoIterator<Item = &'a str>,
		features: ash::vk::PhysicalDeviceFeatures, physical_device: PhysicalDevice,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<(Vrc<Self>, Vec<Vrc<Queue>>), error::DeviceError> {
		let queues = queues.as_ref();
		let queue_create_infos: Vec<_> = queues
			.iter()
			.map(|q| {
				DeviceQueueCreateInfo::builder()
					.flags(q.flags)
					.queue_family_index(q.queue_family_index)
					.queue_priorities(q.queue_priorities.as_ref())
					.build()
			})
			.collect();

		let cstr_layers = layers.into_iter().map(CString::new).collect::<Result<Vec<_>, _>>()?;
		let ptr_layers: Vec<*const c_char> = cstr_layers.iter().map(|cstr| cstr.as_ptr()).collect();

		let cstr_extensions =
			extensions.into_iter().map(CString::new).collect::<Result<Vec<_>, _>>()?;
		let ptr_extensions: Vec<*const c_char> =
			cstr_extensions.iter().map(|cstr| cstr.as_ptr()).collect();

		log::debug!(
			"Device create info {:#?} {:#?} {:#?} {:#?} {:#?}",
			queues,
			queue_create_infos,
			cstr_layers,
			cstr_extensions,
			features
		);
		let create_info = ash::vk::DeviceCreateInfo::builder()
			.queue_create_infos(&queue_create_infos)
			.enabled_layer_names(ptr_layers.as_slice())
			.enabled_extension_names(ptr_extensions.as_slice())
			.enabled_features(&features)
			.build();

		unsafe {
			Device::from_create_info(instance, create_info, physical_device, host_memory_allocator)
		}
	}

	/// Creates a new `Device` from existing `DeviceCreateInfo`
	///
	/// ### Safety
	///
	/// See https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDeviceCreateInfo.html
	pub unsafe fn from_create_info(
		instance: Vrc<Instance>, create_info: DeviceCreateInfo, physical_device: PhysicalDevice,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<(Vrc<Self>, Vec<Vrc<Queue>>), error::DeviceError> {
		let allocation_callbacks: Option<AllocationCallbacks> = host_memory_allocator.into();

		log::debug!(
			"Creating device with {:#?} {:#?} {:#?}",
			physical_device,
			create_info,
			allocation_callbacks
		);
		let device = instance.create_device(
			*physical_device,
			&create_info,
			allocation_callbacks.as_ref()
		)?;

		let elf = Vrc::new(Device { instance, device, physical_device, allocation_callbacks });
		let queues = elf.get_created_queues(create_info);

		Ok(
			(elf, queues)
		)
	}

	unsafe fn get_created_queues(self: &Vrc<Self>, create_info: DeviceCreateInfo) -> Vec<Vrc<Queue>> {
		let num = create_info.queue_create_info_count as usize;
		let mut result = Vec::with_capacity(num);

		for family in 0 .. num as isize {
			let info = &*create_info.p_queue_create_infos.offset(family);

			for index in 0 .. info.queue_count {
				result.push(
					Vrc::new(
						Queue::from_device(
							self.clone(),
							info.flags,
							info.queue_family_index,
							index
						)
					)
				);
			}
		}

		result
	}

	pub fn physical_device(&self) -> &PhysicalDevice {
		&self.physical_device
	}
}
impl Deref for Device {
	type Target = ash::Device;

	fn deref(&self) -> &Self::Target { &self.device }
}
impl Drop for Device {
	fn drop(&mut self) {
		unsafe {
			// Ensure all work is done
			self.device.device_wait_idle().expect("Could not wait for device");

			self.device.destroy_device(self.allocation_callbacks.as_ref());
		}
	}
}
impl Debug for Device {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Device")
			.field("instance", &self.instance)
			.field("device", &crate::util::fmt::format_handle(self.device.handle()))
			.finish()
	}
}