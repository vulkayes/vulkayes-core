use std::fmt::{Debug, Display, Error, Formatter};

macro_rules! log_trace_common {
	(
		$title: literal,
		$(
			$log_item: expr
		),*
	) => {
		log::trace!(
			concat!(
				$title,
				$(
					concat!(" ", stringify!($log_item), " = ", "{:?}")
				),*
			),
			$(
				$log_item
			),*
		)
	}
}

/// Formats Vulkan handle as `<ObjectType $raw>`.
pub fn format_handle<H: ash::vk::Handle>(handle: H) -> impl Debug + Display {
	struct Inner {
		ty: ash::vk::ObjectType,
		raw: u64
	}
	impl Debug for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
			write!(f, "<{:?} 0x{:x}>", self.ty, self.raw)
		}
	}
	impl Display for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
			write!(f, "<{:?} 0x{:x}>", self.ty, self.raw)
		}
	}

	Inner {
		ty: H::TYPE,
		raw: handle.as_raw()
	}
}

#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct VkVersion(pub u32);
impl VkVersion {
	pub fn new(major: u32, minor: u32, patch: u32) -> Self {
		VkVersion(ash::vk::make_version(major, minor, patch))
	}
}
impl Debug for VkVersion {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		<VkVersion as Display>::fmt(self, f)
	}
}
impl Display for VkVersion {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(
			f,
			"v{}.{}.{}",
			ash::vk::version_major(self.0),
			ash::vk::version_minor(self.0),
			ash::vk::version_patch(self.0)
		)
	}
}
impl From<u32> for VkVersion {
	fn from(v: u32) -> Self {
		VkVersion(v)
	}
}

/// Formats `[u8; 16]` as canonical `xxxxxxxx-xxxx-Mxxx-Nxxx-xxxxxxxxxxxx`.
pub fn format_uuid(uuid: [u8; 16]) -> impl Debug + Display {
	struct Inner {
		uuid: [u8; 16]
	}
	impl Debug for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
			write!(
				f,
				"{:0>2x}{:0>2x}{:0>2x}{:0>2x}-{:0>2x}{:0>2x}-{:0>2x}{:0>2x}-{:0>2x}{:0>2x}-{:0>2x}{:0>2x}{:0>2x}{:0>2x}{:0>2x}{:0>2x}",
				self.uuid[0], self.uuid[1], self.uuid[2], self.uuid[3],
				self.uuid[4], self.uuid[5],
				self.uuid[6], self.uuid[7],
				self.uuid[8], self.uuid[9],
				self.uuid[10], self.uuid[11], self.uuid[12], self.uuid[13], self.uuid[14], self.uuid[15]
			)
		}
	}
	impl Display for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
			write!(
				f,
				"{:x}{:x}{:x}{:x}-{:x}{:x}-{:x}{:X}-{:x}{:x}-{:x}{:x}{:x}{:x}{:x}{:x}",
				self.uuid[0],
				self.uuid[1],
				self.uuid[2],
				self.uuid[3],
				self.uuid[4],
				self.uuid[5],
				self.uuid[6],
				self.uuid[7],
				self.uuid[8],
				self.uuid[9],
				self.uuid[10],
				self.uuid[11],
				self.uuid[12],
				self.uuid[13],
				self.uuid[14],
				self.uuid[15]
			)
		}
	}

	Inner { uuid }
}
