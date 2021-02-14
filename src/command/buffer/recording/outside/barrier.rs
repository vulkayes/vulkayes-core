use std::num::NonZeroU64;

use ash::vk;

use ash::version::DeviceV1_0;

use crate::prelude::{HasHandle, Transparent, Buffer, Image, Queue, ImageLayoutFinal, ImageSubresourceRange};

// salmon
vk_builder_wrap! {
	pub struct MemoryBarrier {
		builder: vk::MemoryBarrierBuilder<'static> => vk::MemoryBarrier
	}
	impl {
		pub fn new(
			source_access: vk::AccessFlags,
			destination_access: vk::AccessFlags
		) -> Self {
			MemoryBarrier {
				builder: vk::MemoryBarrier::builder()
					.src_access_mask(source_access)
					.dst_access_mask(destination_access)
			}
		}
	}
}

vk_builder_wrap! {
	pub struct BufferMemoryBarrier ['a] {
		builder: vk::BufferMemoryBarrierBuilder<'a> => vk::BufferMemoryBarrier
	}
	impl ['a] {
		pub fn new(
			buffer: &'a Buffer,
			offset: u64,
			size: NonZeroU64,
			source_access: vk::AccessFlags,
			destination_access: vk::AccessFlags
		) -> Self {
			debug_assert!(
				offset + size.get() <= buffer.size().get()
			);

			BufferMemoryBarrier {
				builder: vk::BufferMemoryBarrier::builder()
					.buffer(buffer.handle())
					.offset(offset)
					.size(size.get())
					.src_access_mask(source_access)
					.dst_access_mask(destination_access)
					.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
					.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			}
		}

		pub fn queue_release(
			_buffer: &'a Buffer,
			_offset: u64,
			_size: NonZeroU64,
			_source_access: vk::AccessFlags,
			_source_queue: &Queue
		) -> Self {
			todo!("need sharing mode access on buffer")
		}

		pub fn queue_acquire(
			_buffer: &'a Buffer,
			_offset: u64,
			_size: NonZeroU64,
			_destination_access: vk::AccessFlags,
			_destination_queue: &Queue
		) -> Self {
			todo!("need sharing mode access on buffer")
		}
	}
}

vk_builder_wrap! {
	pub struct ImageMemoryBarrier ['a] {
		builder: vk::ImageMemoryBarrierBuilder<'a> => vk::ImageMemoryBarrier
	}
	impl ['a] {
		pub fn new(
			image: &'a Image,
			subresource_range: ImageSubresourceRange,
			old_layout: vk::ImageLayout,
			new_layout: ImageLayoutFinal,
			source_access: vk::AccessFlags,
			destination_access: vk::AccessFlags,
		) -> Self {
			debug_assert!(
				subresource_range.mipmap_levels_base + subresource_range.mipmap_levels.get() <= image.size().mipmap_levels().get()
			);
			debug_assert!(
				subresource_range.array_layers_base + subresource_range.array_layers.get() <= image.size().array_layers().get()
			);

			ImageMemoryBarrier {
				builder: vk::ImageMemoryBarrier::builder()
					.image(image.handle())
					.subresource_range(
						vk::ImageSubresourceRangeBuilder::from(
							subresource_range
						).build()
					)
					.old_layout(old_layout)
					.new_layout(new_layout.into())
					.src_access_mask(source_access)
					.dst_access_mask(destination_access)
					.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
					.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			}
		}

		pub fn queue_release(
			_image: &'a Image,
			_subresource_range: ImageSubresourceRange,
			_old_layout: vk::ImageLayout,
			_new_layout: ImageLayoutFinal,
			_source_access: vk::AccessFlags,
			_source_queue: &Queue
		) -> Self {
			todo!("need sharing mode access on image")
		}

		pub fn queue_acquire(
			_image: &'a Image,
			_subresource_range: ImageSubresourceRange,
			_old_layout: vk::ImageLayout,
			_new_layout: ImageLayoutFinal,
			_destination_access: vk::AccessFlags,
			_destination_queue: &Queue
		) -> Self {
			todo!("need sharing mode access on image")
		}
	}
}

impl<'a> super::super::CommandBufferRecordingLockOutsideRenderPass<'a> {
	pub fn pipeline_barrier<'b, 'i>(
		&self,
		source_stages: vk::PipelineStageFlags,
		destination_stages: vk::PipelineStageFlags,
		memory_barriers: impl AsRef<[MemoryBarrier]>,
		buffer_memory_barriers: impl AsRef<[BufferMemoryBarrier<'b>]>,
		image_memory_barriers: impl AsRef<[ImageMemoryBarrier<'i>]>,
	) {
		log_trace_common!(
			"Pipeline barrier:",
			crate::util::fmt::format_handle(self.handle()),
			source_stages,
			destination_stages,
			memory_barriers.as_ref(),
			buffer_memory_barriers.as_ref(),
			image_memory_barriers.as_ref()
		);
		unsafe {
			self.device().cmd_pipeline_barrier(
				self.handle(),
				source_stages,
				destination_stages,
				vk::DependencyFlags::empty(),
				Transparent::transmute_slice_twice(memory_barriers.as_ref()),
				Transparent::transmute_slice_twice(buffer_memory_barriers.as_ref()),
				Transparent::transmute_slice_twice(image_memory_barriers.as_ref())
			)
		}
	}
}