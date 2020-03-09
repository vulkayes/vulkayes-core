use std::ops::Deref;
use std::fmt::{Debug, self};

use ash::vk;
use ash::version::DeviceV1_0;

use crate::Vrc;
use crate::device::Device;

pub struct Image {
	device: Vrc<Device>,
	image: vk::Image,

	format: vk::Format,
	size: super::ImageSize,

	// TODO: Allocation callbacks?
}
impl Image {
	/// Crates a new `Image` from existing `VkImage`.
	///
	/// ### Safety
	///
	/// * `image` must have been crated from the `device`.
	/// * All parameters must match the parameters used when creating the image.
	pub unsafe fn from_existing(
		device: Vrc<Device>,
		image: vk::Image,
		format: vk::Format,
		size: super::ImageSize
	) -> Self {
		log::trace!(
			"Creating Image from existing handle {:#?} {:#?} {:#?} {:#?}",
			device,
			crate::util::fmt::format_handle(image),
			format,
			size
		);
		Image {
			device,
			image,
			format,
			size
		}
	}

	pub fn size(&self) -> super::ImageSize {
		self.size
	}

	pub fn format(&self) -> vk::Format {
		self.format
	}

	pub fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for Image {
		type Target = vk::Image { image }
	}
}
impl Drop for Image {
	fn drop(&mut self) {
		unsafe {
			self.device.destroy_image(self.image, None);
		}
	}
}
impl Debug for Image {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Image")
			.field("device", &self.device)
			.field("image", &crate::util::fmt::format_handle(self.image))
			.field("format", &self.format)
			.field("size", &self.size)
			.finish()
	}
}