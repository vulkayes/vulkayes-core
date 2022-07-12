use std::{num::NonZeroU64, ops::Deref, ptr::NonNull};

use ash::vk;

use super::{
	allocator::{BufferMemoryAllocator, ImageMemoryAllocator},
	DeviceMemoryAllocation
};
use crate::{device::Device, physical_device::enumerate::PhysicalDeviceMemoryProperties, prelude::Vrc};

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

	fn find_memory_index(&self, requirements: vk::MemoryRequirements, required_flags: vk::MemoryPropertyFlags) -> Result<u32, AllocationError> {
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

	fn allocate(&self, info: impl Deref<Target = vk::MemoryAllocateInfo>) -> Result<DeviceMemoryAllocation, AllocationError> {
		let memory = unsafe { self.device.allocate_memory(&info, None)? };
		let size = unsafe { NonZeroU64::new_unchecked(info.allocation_size) };

		Ok(unsafe {
			DeviceMemoryAllocation::new(
				self.device.clone(),
				memory,
				0,
				size,
				Box::new(|device, memory, offset, size| {
					let ptr = device.map_memory(
						memory,
						offset,
						size.get(),
						vk::MemoryMapFlags::empty()
					)? as *mut u8;
					debug_assert_ne!(ptr, std::ptr::null_mut());

					let slice_ptr = std::slice::from_raw_parts_mut(ptr, size.get() as usize) as *mut [u8];
					Ok(NonNull::new_unchecked(slice_ptr))
				}),
				Box::new(|device, memory, _, _, _| device.unmap_memory(memory)),
				Box::new(|device, memory, _, _| device.free_memory(memory, None))
			)
		})
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
unsafe impl ImageMemoryAllocator for NaiveDeviceMemoryAllocator {
	type AllocationRequirements = vk::MemoryPropertyFlags;
	type Error = AllocationError;

	fn allocate(&self, image: vk::Image, required_flags: Self::AllocationRequirements) -> Result<DeviceMemoryAllocation, Self::Error> {
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
		self.allocate(alloc_info)
	}
}
unsafe impl BufferMemoryAllocator for NaiveDeviceMemoryAllocator {
	type AllocationRequirements = vk::MemoryPropertyFlags;
	type Error = AllocationError;

	fn allocate(&self, buffer: vk::Buffer, required_flags: Self::AllocationRequirements) -> Result<DeviceMemoryAllocation, Self::Error> {
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
		self.allocate(alloc_info)
	}
}
