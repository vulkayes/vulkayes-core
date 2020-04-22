use std::{fmt, num::NonZeroU64, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::{prelude::HostMemoryAllocator, prelude::Buffer, prelude::Vrc, prelude::HasHandle};

pub struct BufferView {
	buffer: Vrc<Buffer>,
	view: vk::BufferView,

	format: vk::Format,

	offset: vk::DeviceSize,
	range: NonZeroU64,

	host_memory_allocator: HostMemoryAllocator
}
impl BufferView {
	pub fn new(
		buffer: Vrc<Buffer>,
		format: vk::Format,
		offset: vk::DeviceSize,
		range: NonZeroU64,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, super::error::BufferViewError> {
		let create_info = vk::BufferViewCreateInfo::builder()
			.buffer(buffer.handle())
			.format(format)
			.offset(offset)
			.range(range.get());

		unsafe { Self::from_create_info(buffer, create_info, host_memory_allocator) }
	}

	/// ### Safety
	///
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateBufferView.html>.
	/// `buffer` must be the same buffer as the one in the `create_info`.
	pub unsafe fn from_create_info(
		buffer: Vrc<Buffer>,
		create_info: impl Deref<Target = vk::BufferViewCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, super::error::BufferViewError> {
		let c_info = create_info.deref();

		log_trace_common!("Create buffer view:", buffer, c_info, host_memory_allocator);
		let view = buffer
			.device()
			.create_buffer_view(c_info, host_memory_allocator.as_ref())?;

		let format = c_info.format;
		let offset = c_info.offset;
		let range = NonZeroU64::new(c_info.range)
			.unwrap_or(NonZeroU64::new_unchecked(buffer.size().get() - offset));

		Ok(Vrc::new(BufferView {
			buffer,
			view,

			format,

			offset,
			range,

			host_memory_allocator
		}))
	}

	pub const fn buffer(&self) -> &Vrc<Buffer> {
		&self.buffer
	}

	pub const fn format(&self) -> vk::Format {
		self.format
	}

	pub const fn offset(&self) -> vk::DeviceSize {
		self.offset
	}

	pub const fn range(&self) -> NonZeroU64 {
		self.range
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::BufferView>, Deref, Borrow, Eq, Hash, Ord for BufferView {
		target ={ view }
	}
}
impl Drop for BufferView {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.buffer
				.device()
				.destroy_buffer_view(self.view, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for BufferView {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("BufferView")
			.field("buffer", &self.buffer)
			.field("view", &self.safe_handle())
			.field("format", &self.format)
			.field("offset", &self.offset)
			.field("range", &self.range)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
