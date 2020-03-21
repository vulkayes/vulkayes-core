use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::{device::Device, physical_device::enumerate::PhysicalDeviceMemoryProperties, Vrc};

use super::{BufferMemoryAllocator, DeviceMemoryAllocation, ImageMemoryAllocator};

vk_result_error! {
	#[derive(Debug)]
	pub enum AllocationError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_TOO_MANY_OBJECTS,
			ERROR_INVALID_EXTERNAL_HANDLE,
			ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS
		}

		#[error("Suitable memory type could not be found")]
		NoSuitableMemoryType,
	}
}

pub struct NaiveDeviceMemoryAllocation {
	device: Vrc<Device>,
	memory: vk::DeviceMemory
}
impl Deref for NaiveDeviceMemoryAllocation {
	type Target = vk::DeviceMemory;

	fn deref(&self) -> &Self::Target {
		&self.memory
	}
}
unsafe impl DeviceMemoryAllocation for NaiveDeviceMemoryAllocation {
	fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	fn bind_offset(&self) -> vk::DeviceSize {
		0
	}
}
impl Drop for NaiveDeviceMemoryAllocation {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe { self.device.free_memory(self.memory, None) }
	}
}
impl fmt::Debug for NaiveDeviceMemoryAllocation {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("NaiveDeviceMemoryAllocation")
			.field("memory", &crate::util::fmt::format_handle(self.memory))
			.finish()
	}
}

/// Simple device memory allocator.
///
/// Allocates new memory for each request. This allocator is useful when prototyping or debugging,
/// but not in bigger production applications.
#[derive(Debug, Clone)]
pub struct NaiveDeviceMemoryAllocator {
	device: Vrc<Device>,
	properties: PhysicalDeviceMemoryProperties
}
impl NaiveDeviceMemoryAllocator {
	pub fn new(device: Vrc<Device>) -> Self {
		let properties = device.physical_device().memory_properties();

		NaiveDeviceMemoryAllocator { device, properties }
	}

	fn find_memory_index(
		&self,
		requirements: vk::MemoryRequirements,
		required_flags: vk::MemoryPropertyFlags
	) -> Result<u32, AllocationError> {
		for (index, memory_type) in self.properties.memory_types.iter().enumerate() {
			// If this type is in the mask of allowed types
			if requirements.memory_type_bits & (1 << index as u32) != 0 {
				// and contains all the required flags
				if memory_type.property_flags.contains(required_flags) {
					return Ok(index as u32)
				}
			}
		}

		Err(AllocationError::NoSuitableMemoryType)
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
unsafe impl ImageMemoryAllocator for NaiveDeviceMemoryAllocator {
	type Allocation = NaiveDeviceMemoryAllocation;
	type AllocationRequirements = vk::MemoryPropertyFlags;
	type Error = AllocationError;

	fn allocate(
		&mut self,
		image: vk::Image,
		required_flags: Self::AllocationRequirements
	) -> Result<Self::Allocation, Self::Error> {
		let memory_requirements = unsafe { self.device.get_image_memory_requirements(image) };
		let memory_index = self.find_memory_index(memory_requirements, required_flags)?;

		let alloc_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(memory_requirements.size)
			.memory_type_index(memory_index);

		log_trace_common!(
			"Allocating image memory:",
			crate::util::fmt::format_handle(image),
			required_flags,
			alloc_info.deref()
		);
		let memory = unsafe { self.device.allocate_memory(&alloc_info, None)? };

		Ok(NaiveDeviceMemoryAllocation {
			device: self.device.clone(),
			memory
		})
	}
}
unsafe impl BufferMemoryAllocator for NaiveDeviceMemoryAllocator {
	type Allocation = NaiveDeviceMemoryAllocation;
	type AllocationRequirements = vk::MemoryPropertyFlags;
	type Error = AllocationError;

	fn allocate(
		&mut self,
		buffer: vk::Buffer,
		required_flags: Self::AllocationRequirements
	) -> Result<Self::Allocation, Self::Error> {
		let memory_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };
		let memory_index = self.find_memory_index(memory_requirements, required_flags)?;

		let alloc_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(memory_requirements.size)
			.memory_type_index(memory_index);


		log_trace_common!(
			"Allocating buffer memory:",
			crate::util::fmt::format_handle(buffer),
			required_flags,
			alloc_info.deref()
		);
		let memory = unsafe { self.device.allocate_memory(&alloc_info, None)? };

		Ok(NaiveDeviceMemoryAllocation {
			device: self.device.clone(),
			memory
		})
	}
}
