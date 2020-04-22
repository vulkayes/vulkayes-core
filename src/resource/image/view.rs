use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::{prelude::HostMemoryAllocator, prelude::Vrc, prelude::HasHandle};

use super::params::{ImageSize, ImageSubresourceRange};

pub struct ImageView {
	image: super::MixedDynImage,
	view: vk::ImageView,

	format: vk::Format,
	component_mapping: vk::ComponentMapping,

	subresource_range: ImageSubresourceRange,
	subresource_image_size: ImageSize,

	host_memory_allocator: HostMemoryAllocator
}
impl ImageView {
	pub fn new(
		image: super::MixedDynImage,
		view_range: super::params::ImageViewRange,
		format: Option<vk::Format>,
		component_mapping: vk::ComponentMapping,
		view_aspect: vk::ImageAspectFlags,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, super::error::ImageViewError> {
		let subresource_slice: super::params::ImageSubresourceSlice = view_range.into();

		let create_info = vk::ImageViewCreateInfo::builder()
			.image(image.handle())
			.view_type(subresource_slice.view_type)
			.format(format.unwrap_or(image.format()))
			.components(component_mapping)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: view_aspect,
				base_mip_level: subresource_slice.mipmap_levels_base,
				level_count: subresource_slice.mipmap_levels.get(),
				base_array_layer: subresource_slice.array_layers_base,
				layer_count: subresource_slice.array_layers.get()
			});

		unsafe { Self::from_create_info(image, create_info, host_memory_allocator) }
	}

	/// Creates a new `ImageView` from create info.
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateImageView.html>.
	pub unsafe fn from_create_info(
		image: super::MixedDynImage,
		create_info: impl Deref<Target = vk::ImageViewCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, super::error::ImageViewError> {
		let c_info = create_info.deref();

		log_trace_common!("Create image view:", image, c_info, host_memory_allocator);
		let view = image
			.device()
			.create_image_view(c_info, host_memory_allocator.as_ref())?;

		let subresource_range = ImageSubresourceRange::from_image_view_create_info(c_info);
		let subresource_image_size = {
			let image_type = match create_info.view_type {
				vk::ImageViewType::TYPE_1D | vk::ImageViewType::TYPE_1D_ARRAY => {
					vk::ImageType::TYPE_1D
				}
				vk::ImageViewType::TYPE_2D
				| vk::ImageViewType::TYPE_2D_ARRAY
				| vk::ImageViewType::CUBE
				| vk::ImageViewType::CUBE_ARRAY => vk::ImageType::TYPE_2D,
				vk::ImageViewType::TYPE_3D => vk::ImageType::TYPE_3D,
				_ => unreachable!()
			};

			let image_size = image.size();
			ImageSize::new(
				image_type,
				image_size.width(),
				image_size.height(),
				image_size.depth(),
				subresource_range.array_layers,
				subresource_range.mipmap_levels
			)
		};

		Ok(Vrc::new(ImageView {
			image,
			view,

			format: c_info.format,
			component_mapping: c_info.components,

			subresource_range,
			subresource_image_size,

			host_memory_allocator
		}))
	}

	pub const fn image(&self) -> &super::MixedDynImage {
		&self.image
	}

	pub const fn format(&self) -> vk::Format {
		self.format
	}

	pub const fn component_mapping(&self) -> vk::ComponentMapping {
		self.component_mapping
	}

	pub const fn subresource_range(&self) -> ImageSubresourceRange {
		self.subresource_range
	}

	pub const fn subresource_image_size(&self) -> ImageSize {
		self.subresource_image_size
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::ImageView>, Deref, Borrow, Eq, Hash, Ord for ImageView {
		target = { view }
	}
}
impl Drop for ImageView {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.image
				.device()
				.destroy_image_view(self.view, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for ImageView {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("ImageView")
			.field("image", &self.image)
			.field("view", &self.safe_handle())
			.field("format", &self.format)
			.field("component_mapping", &self.component_mapping)
			.field("subresource_range", &self.subresource_range)
			.field("subresource_image_size", &self.subresource_image_size)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
