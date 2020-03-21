use std::{
	fmt::{self, Debug},
	num::NonZeroU32,
	ops::Deref
};

use ash::{version::DeviceV1_0, vk};

use crate::{
	device::Device,
	memory::{
		device::{never::NeverMemoryAllocation, DeviceMemoryAllocation, ImageMemoryAllocator},
		host::HostMemoryAllocator
	},
	queue::sharing_mode::SharingMode,
	Vrc
};

pub mod error;
pub mod params;
pub mod size;

pub struct Image<Mem: DeviceMemoryAllocation = NeverMemoryAllocation> {
	device: Vrc<Device>,
	image: vk::Image,
	memory: Option<Mem>, // This field is optional, but the option is also used in `Drop`

	format: vk::Format,
	size: size::ImageSize,

	host_memory_allocator: HostMemoryAllocator
}
impl<Mem: DeviceMemoryAllocation> Image<Mem> {
	pub fn new<A: ImageMemoryAllocator<Allocation = Mem>>(
		device: Vrc<Device>,
		format: vk::Format,
		size_info: params::ImageSizeInfo,
		tiling_and_layout: params::ImageTilingAndLayout,
		usage: vk::ImageUsageFlags,
		sharing_mode: SharingMode<impl AsRef<[u32]>>,
		allocator: Option<&mut A>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::ImageError<A::Error>> {
		if cfg!(feature = "runtime_implicit_validations") {
			if usage.is_empty() {
				return Err(error::ImageError::UsageEmpty)
			}
		}

		let (size, mipmaps, samples, flags) = size_info.into();
		let (tiling, layout) = tiling_and_layout.into();

		let mipmap_levels = {
			let maybe_number: Option<NonZeroU32> = mipmaps.into();
			maybe_number.unwrap_or_else(|| {
				// TODO: This should consult `VkImageFormatProperties2 imageCreateImageFormatPropertiesList[]` to check the `imageCreateMaxMipLevels`
				size.complete_mipmap_chain_mipmaps()
			})
		};

		let create_info = vk::ImageCreateInfo::builder()
			.flags(flags)
			.image_type(size.image_type())
			.format(format)
			.extent(size.into())
			.mip_levels(mipmap_levels.get())
			.array_layers(size.array_layers().get())
			.samples(samples)
			.tiling(tiling)
			.usage(usage)
			.sharing_mode(sharing_mode.sharing_mode())
			.queue_family_indices(sharing_mode.indices())
			.initial_layout(layout);

		unsafe { Self::from_create_info(device, create_info, allocator, host_memory_allocator) }
	}

	/// Creates a new `Image` from existing `ImageCreateInfo`
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateImage.html>.
	pub unsafe fn from_create_info<A: ImageMemoryAllocator<Allocation = Mem>>(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::ImageCreateInfo>,
		allocator: Option<&mut A>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::ImageError<A::Error>> {
		log_trace_common!(
			"Create image:",
			device,
			create_info.deref(),
			allocator,
			host_memory_allocator
		);

		let c_info = create_info.deref();
		let image = device.create_image(c_info, host_memory_allocator.as_ref())?;

		let memory = allocator
			.map(|alloc| alloc.allocate(image))
			.transpose()
			.map_err(|err| error::ImageError::AllocationError(err))?;
		if cfg!(feature = "runtime_implicit_validations") {
			if let Some(ref memory) = memory {
				if memory.device() != &device {
					return Err(error::ImageError::MemoryDeviceMismatch)
				}
			}
		}

		if let Some(ref memory) = memory {
			device.bind_image_memory(image, *memory.deref(), memory.bind_offset())?;
		}

		let width = NonZeroU32::new(c_info.extent.width).expect("width must be non zero");
		let height = NonZeroU32::new(c_info.extent.width).expect("height must be non zero");
		let depth = NonZeroU32::new(c_info.extent.width).expect("depth must be non zero");
		let array_layers =
			NonZeroU32::new(c_info.array_layers).expect("array layers must be non zero");

		let size = match c_info.image_type {
			vk::ImageType::TYPE_1D => size::ImageSize::new_1d(width, array_layers).into(),
			vk::ImageType::TYPE_2D => size::ImageSize::new_2d(width, height, array_layers).into(),
			vk::ImageType::TYPE_3D => size::ImageSize::new_3d(width, height, depth).into(),
			_ => unreachable!()
		};

		Ok(Vrc::new(Image {
			device,
			image,
			memory,

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
		memory: Option<Mem>,
		format: vk::Format,
		size: size::ImageSize,
		host_memory_allocator: HostMemoryAllocator
	) -> Self {
		log_trace_common!(
			"Creating Image from existing handle:",
			device,
			crate::util::fmt::format_handle(image),
			format,
			size
		);
		Image {
			device,
			image,
			memory,
			format,
			size,
			host_memory_allocator
		}
	}

	// TODO: The following getters cannot be const because of generics
	pub fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub fn size(&self) -> size::ImageSize {
		self.size
	}

	pub fn format(&self) -> vk::Format {
		self.format
	}

	pub fn memory(&self) -> &Option<Mem> {
		&self.memory
	}
}
impl_common_handle_traits! {
	impl [M: DeviceMemoryAllocation] Deref, PartialEq, Eq, Hash for Image<M> {
		type Target = vk::Image { image }
	}
}
impl<M: DeviceMemoryAllocation> Drop for Image<M> {
	fn drop(&mut self) {
		unsafe {
			self.device
				.destroy_image(self.image, self.host_memory_allocator.as_ref());
		}
	}
}
impl<M: DeviceMemoryAllocation> Debug for Image<M> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Image")
			.field("device", &self.device)
			.field("image", &crate::util::fmt::format_handle(self.image))
			.field(
				"memory",
				&self
					.memory
					.as_ref()
					.map(|m| crate::util::fmt::format_handle(*m.deref()))
			)
			.field("format", &self.format)
			.field("size", &self.size)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
