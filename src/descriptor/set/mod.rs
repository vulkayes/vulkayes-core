use std::{fmt::Debug, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::prelude::{DescriptorPool, DescriptorSetLayout, Transparent, Vrc, Vutex};

use super::error::DescriptorSetError;

pub mod update;

pub struct DescriptorSet {
	pool: Vrc<DescriptorPool>,
	descriptor_set: Vutex<vk::DescriptorSet>
}
impl DescriptorSet {
	pub fn new(
		pool: Vrc<DescriptorPool>,
		layout: Vrc<DescriptorSetLayout>
	) -> Result<Vrc<Self>, DescriptorSetError> {
		let [raw] = pool.allocate_descriptor_set([&layout])?;

		Ok(Vrc::new(unsafe { Self::from_existing(pool, raw) }))
	}

	pub fn new_multiple(
		pool: Vrc<DescriptorPool>,
		layouts: Vec<Vrc<DescriptorSetLayout>>
	) -> Result<Vec<Vrc<Self>>, DescriptorSetError> {
		let raw = pool.allocate_descriptor_sets(
			layouts.iter().map(|l| l.deref())
		)?;

		let sets: Vec<_> = raw.into_iter().map(
			|descriptor_set| {
				Vrc::new(unsafe { Self::from_existing(pool.clone(), descriptor_set) })
			}
		).collect();

		Ok(sets)
	}

	/// ### Safety
	///
	/// * `descriptor_set` must be a valid handle allocated from `pool`.
	pub unsafe fn from_existing(
		pool: Vrc<DescriptorPool>,
		descriptor_set: vk::DescriptorSet
	) -> Self {
		log_trace_common!(
			"Creating DescriptorSet from existing handle:",
			pool,
			crate::util::fmt::format_handle(descriptor_set)
		);

		Self {
			pool,
			descriptor_set: Vutex::new(descriptor_set)
		}
	}

	pub fn update<'a>(
		&self,
		writes: &[update::DescriptorSetWrite<'a>],
		copies: &[update::DescriptorSetCopy<'a>]
	) {
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
impl Debug for DescriptorSet {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("DescriptorSet")
			.field("pool", &self.pool)
			.field("descriptor_set", &self.descriptor_set)
			.finish()
	}
}
