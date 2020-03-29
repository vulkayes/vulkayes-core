use std::{
	fmt,
	ops::Deref
};

use ash::{version::DeviceV1_0, vk};

use crate::{
	device::Device,
	memory::{
		device::{DeviceMemoryAllocation, ImageMemoryAllocator},
		host::HostMemoryAllocator
	},
	queue::sharing_mode::SharingMode,
	Vrc
};

use super::{params, error};

pub struct Image {
	device: Vrc<Device>,
	image: vk::Image,
	// Dynamic dispatch doesn't hurt because the memory is not accessed often, it only needs to be kept alive
	memory: Option<DeviceMemoryAllocation>,

	usage: vk::ImageUsageFlags,
	format: vk::Format,
	size: params::ImageSize,
	// TODO: Tiling and sharing mode + indices?

	host_memory_allocator: HostMemoryAllocator
}
impl Image {
	pub fn new<A: ImageMemoryAllocator>(
		device: Vrc<Device>,
		format: vk::Format,
		size_info: params::ImageSizeInfo,
		tiling_and_layout: params::ImageTilingAndLayout,
		usage: vk::ImageUsageFlags,
		sharing_mode: SharingMode<impl AsRef<[u32]>>,
		allocator_param: params::AllocatorParams<A>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::ImageError<A::Error>> {
		if cfg!(feature = "runtime_implicit_validations") {
			if usage.is_empty() {
				return Err(error::ImageError::UsageEmpty)
			}
		}

		let (size, samples, flags) = size_info.into();
		let (tiling, layout) = tiling_and_layout.into();

		let create_info = vk::ImageCreateInfo::builder()
			.flags(flags)
			.image_type(size.image_type())
			.format(format)
			.extent(size.into())
			.mip_levels(size.mipmap_levels().get())
			.array_layers(size.array_layers().get())
			.samples(samples)
			.tiling(tiling)
			.usage(usage)
			.sharing_mode(sharing_mode.sharing_mode())
			.queue_family_indices(sharing_mode.indices())
			.initial_layout(layout);

		unsafe { Self::from_create_info(device, create_info, allocator_param, host_memory_allocator) }
	}

	/// Creates a new `Image` from existing `ImageCreateInfo`
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateImage.html>.
	pub unsafe fn from_create_info<A: ImageMemoryAllocator>(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::ImageCreateInfo>,
		allocator_params: params::AllocatorParams<A>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::ImageError<A::Error>> {
		let c_info = create_info.deref();

		log_trace_common!(
			"Create image:",
			device,
			c_info,
			allocator_params,
			host_memory_allocator
		);
		let image = device.create_image(c_info, host_memory_allocator.as_ref())?;

		let memory = match allocator_params {
			params::AllocatorParams::Some { allocator, requirements } => {
				let memory = allocator
					.allocate(image, requirements)
					.map_err(error::ImageError::AllocationError)?;

				if cfg!(feature = "runtime_implicit_validations") {
					if memory.device() != &device {
						return Err(error::ImageError::MemoryDeviceMismatch)
					}
				}

				device.bind_image_memory(image, *memory.deref(), memory.bind_offset())?;
				Some(Vrc::new(memory) as Vrc<_>)
			}
			params::AllocatorParams::None => None
		};

		let size = params::ImageSize::from_image_create_info(c_info);

		Ok(Vrc::new(Image {
			device,
			image,
			memory,

			usage: c_info.usage,
			format: c_info.format,
			size,

			host_memory_allocator
		}))
	}

	/// Crates a new `Image` from existing `VkImage`.
	///
	/// ### Safety
	///
	/// * `image` must have been crated from the `device`.
	/// * `memory` must have been allocated from the `device`.
	/// * All parameters must match the parameters used when creating the image.
	pub unsafe fn from_existing(
		device: Vrc<Device>,
		image: vk::Image,
		memory: Option<DeviceMemoryAllocation>,
		usage: vk::ImageUsageFlags,
		format: vk::Format,
		size: params::ImageSize,
		host_memory_allocator: HostMemoryAllocator
	) -> Self {
		log_trace_common!(
			"Creating Image from existing handle:",
			device,
			crate::util::fmt::format_handle(image),
			memory,
			format,
			size,
			host_memory_allocator
		);

		Image {
			device,
			image,
			memory,
			usage,
			format,
			size,
			host_memory_allocator
		}
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub const fn usage(&self) -> vk::ImageUsageFlags {
		self.usage
	}

	pub const fn size(&self) -> params::ImageSize {
		self.size
	}

	pub const fn format(&self) -> vk::Format {
		self.format
	}

	// TODO: Cannot be const because of Sized
	pub fn memory(&self) -> Option<&DeviceMemoryAllocation> {
		self.memory.as_ref()
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for Image {
		type Target = vk::Image { image }
	}
}
impl Drop for Image {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device
				.destroy_image(self.image, self.host_memory_allocator.as_ref());
		}
	}
}
impl fmt::Debug for Image {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Image")
			.field("device", &self.device)
			.field("image", &crate::util::fmt::format_handle(self.image))
			.field(
				"memory",
				&self.memory.as_ref().map(|m| crate::util::fmt::format_handle(
						*m.deref().deref()
					))
			)
			.field("usage", &self.usage)
			.field("format", &self.format)
			.field("size", &self.size)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
