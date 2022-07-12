use std::{fmt::Debug, ops::Deref};

use ash::vk;

use super::error::DescriptorSetError;
use crate::prelude::{DescriptorPool, DescriptorSetLayout, HasHandle, Transparent, Vrc, Vutex};

pub mod update;

#[derive(Debug)]
pub struct DescriptorSet {
	pool: Vrc<DescriptorPool>,
	// need to keep layout alive for writes to be valid
	layout: Vrc<DescriptorSetLayout>,
	descriptor_set: Vutex<vk::DescriptorSet>
}
impl DescriptorSet {
	pub fn new(pool: Vrc<DescriptorPool>, layout: Vrc<DescriptorSetLayout>) -> Result<Vrc<Self>, DescriptorSetError> {
		let [raw] = pool.allocate_descriptor_set([layout.safe_handle()])?;

		Ok(Vrc::new(unsafe {
			Self::from_existing(pool, layout, raw)
		}))
	}

	/// ### Safety
	///
	/// * `descriptor_set` must be a valid handle allocated from `pool`.
	/// * `descriptor_set` must have been allocated from `layout`.
	pub unsafe fn from_existing(pool: Vrc<DescriptorPool>, layout: Vrc<DescriptorSetLayout>, descriptor_set: vk::DescriptorSet) -> Self {
		log_trace_common!(
			"Creating DescriptorSet from existing handle:",
			pool,
			layout,
			crate::util::fmt::format_handle(descriptor_set)
		);

		Self { pool, layout, descriptor_set: Vutex::new(descriptor_set) }
	}

	pub fn update<'a>(&self, writes: &[update::DescriptorSetWrite<'a>], copies: &[update::DescriptorSetCopy<'a>]) {
		unsafe {
			self.pool.device().update_descriptor_sets(
				Transparent::transmute_slice_twice(writes),
				Transparent::transmute_slice_twice(copies)
			)
		}
	}

	pub const fn pool(&self) -> &Vrc<DescriptorPool> {
		&self.pool
	}

	pub const fn layout(&self) -> &Vrc<DescriptorSetLayout> {
		&self.layout
	}
}
impl_common_handle_traits! {
	impl HasSynchronizedHandle<vk::DescriptorSet>, Deref, Borrow, Eq, Hash, Ord for DescriptorSet {
		target = { descriptor_set }
	}
}
impl Drop for DescriptorSet {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		// TODO: Not all descriptor sets are free-able
		// unsafe { self.pool.free_command_buffers([*lock]) }
	}
}
