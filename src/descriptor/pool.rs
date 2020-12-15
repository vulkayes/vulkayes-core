use std::{fmt, num::NonZeroU32, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::prelude::{DescriptorSetLayout, Device, HostMemoryAllocator, Vrc, Vutex};

use super::error::{DescriptorPoolError, DescriptorSetError};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct DescriptorPoolSize {
	pub descriptor_type: vk::DescriptorType,
	pub count: NonZeroU32
}
impl Into<vk::DescriptorPoolSize> for DescriptorPoolSize {
	fn into(self) -> vk::DescriptorPoolSize {
		vk::DescriptorPoolSize::builder()
			.ty(self.descriptor_type)
			.descriptor_count(self.count.get())
			.build()
	}
}

macro_rules! impl_allocate_command_buffers_array {
	(
		$name: ident, $size: expr, $layouts_name: ident
	) => {
		/// Allocates descriptor sets into fixed-size array.
		///
		/// ### Panic
		///
		/// This function will panic if the pool `Vutex` is poisoned.
		// Const generics can't come fast enough
		pub fn $name(
			&self,
			$layouts_name: [&DescriptorSetLayout; $size]
		) -> Result<[vk::DescriptorSet; $size], DescriptorSetError> {
			use $crate::util::handle::HasHandle;

			let lock = self.pool.lock().expect("vutex poisoned");

			#[cfg(feature = "runtime_implicit_validations")]
			{
				if $size == 0 {
					return Err(DescriptorSetError::LayoutsEmpty)
				}

				if !$crate::util::validations::validate_all_match(
					std::iter::once(&self.device).chain(
						$layouts_name.iter().map(|l| l.device())
					)
				) {
					return Err(DescriptorSetError::DescriptorPoolLayoutsDeviceMismatch)
				}
			}

			let layout_handles: [vk::DescriptorSetLayout; $size] = $crate::seq_macro::seq!(
				N in 0 .. $size {
					[
						#(
							($layouts_name[N]).handle(),
						)*
					]
				}
			);

			let alloc_info = vk::DescriptorSetAllocateInfo::builder()
				.descriptor_pool(*lock)
				.set_layouts(&layout_handles)
			;

			log_trace_common!(
				"Allocating descriptor sets:",
				self,
				crate::util::fmt::format_handle(*lock),
				alloc_info.deref()
			);

			unsafe {
				let mut sets = std::mem::MaybeUninit::<[vk::DescriptorSet; $size]>::uninit();
				let err_code = self.device.fp_v1_0().allocate_descriptor_sets(
					self.device.handle(),
					alloc_info.deref() as *const _,
					sets.as_mut_ptr() as *mut vk::DescriptorSet
				);

				match err_code {
					vk::Result::SUCCESS => Ok(sets.assume_init()),
					_ => Err(DescriptorSetError::from(err_code))
				}
			}
		}
	};

	(
		$layouts_name: ident
		$($name: ident, $size: expr),+
	) => {
		$(
			impl_allocate_command_buffers_array!($name, $size, $layouts_name);
		)+
	}
}

pub struct DescriptorPool {
	device: Vrc<Device>,
	pool: Vutex<vk::DescriptorPool>,

	host_memory_allocator: HostMemoryAllocator
}
impl DescriptorPool {
	impl_allocate_command_buffers_array!(
		layouts
		allocate_descriptor_set, 1,
		allocate_descriptor_sets_2, 2,
		allocate_descriptor_sets_3, 3,
		allocate_descriptor_sets_4, 4,
		allocate_descriptor_sets_5, 5,
		allocate_descriptor_sets_6, 6,
		allocate_descriptor_sets_7, 7,
		allocate_descriptor_sets_8, 8
	);

	pub fn new(
		device: Vrc<Device>,
		flags: vk::DescriptorPoolCreateFlags,
		max_sets: NonZeroU32,
		pool_sizes: impl Iterator<Item = DescriptorPoolSize>,
		max_inline_uniform_bindings: Option<u32>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, DescriptorPoolError> {
		let sizes = collect_iter_faster!(
			pool_sizes.map(|i| Into::<vk::DescriptorPoolSize>::into(i)),
			4
		);

		let create_info = vk::DescriptorPoolCreateInfo::builder()
			.flags(flags)
			.max_sets(max_sets.get())
			.pool_sizes(&sizes);

		unsafe {
			match max_inline_uniform_bindings {
				None => Self::from_create_info(device, create_info, host_memory_allocator),
				Some(bindings) => {
					let mut bindings_info =
						vk::DescriptorPoolInlineUniformBlockCreateInfoEXT::builder()
							.max_inline_uniform_block_bindings(bindings);
					Self::from_create_info(
						device,
						create_info.push_next(&mut bindings_info),
						host_memory_allocator
					)
				}
			}
		}
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateDescriptorPool.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::DescriptorPoolCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, DescriptorPoolError> {
		log_trace_common!(
			"Creating descriptor pool:",
			device,
			create_info.deref(),
			host_memory_allocator
		);
		let pool =
			device.create_descriptor_pool(create_info.deref(), host_memory_allocator.as_ref())?;

		Ok(Vrc::new(Self {
			device,

			pool: Vutex::new(pool),
			host_memory_allocator
		}))
	}

	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.`
	pub fn allocate_descriptor_sets<'a>(
		&self,
		layouts: impl Iterator<Item = &'a DescriptorSetLayout>
	) -> Result<Vec<vk::DescriptorSet>, DescriptorSetError> {
		let lock = self.pool.lock().expect("vutex poisoned");

		#[cfg(feature = "runtime_implicit_validations")]
		let layouts = {
			// This collect dance is unfortunate
			let collected: Vec<_> = layouts.collect();
			if collected.len() == 0 {
				return Err(DescriptorSetError::LayoutsEmpty)
			}

			if !crate::util::validations::validate_all_match(
				std::iter::once(&self.device).chain(collected.iter().map(|l| l.device()))
			) {
				return Err(DescriptorSetError::DescriptorPoolLayoutsDeviceMismatch)
			}

			collected.into_iter()
		};

		let layouts = collect_iter_faster!(layouts.map(|l| *l.deref()), 8);

		let alloc_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*lock)
			.set_layouts(&layouts);

		log_trace_common!(
			"Allocating descriptor sets:",
			self,
			crate::util::fmt::format_handle(*lock),
			alloc_info.deref()
		);

		unsafe {
			self.device
				.allocate_descriptor_sets(alloc_info.deref())
				.map_err(Into::into)
		}
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkFreeDescriptorSets.html>.
	///
	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub unsafe fn free_descriptor_sets(&self, descriptor_sets: impl AsRef<[vk::DescriptorSet]>) {
		let lock = self.pool.lock().expect("vutex poisoned");

		log_trace_common!(
			"Freeing descriptor sets:",
			self,
			crate::util::fmt::format_handle(*lock),
			descriptor_sets.as_ref()
		);

		self.device
			.free_descriptor_sets(*lock, descriptor_sets.as_ref())
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkResetDescriptorPool.html>.
	pub unsafe fn reset(&self) {
		let lock = self.pool.lock().expect("vutex poisoned");

		log_trace_common!(
			"Freeing resetting descriptor pool:",
			self,
			crate::util::fmt::format_handle(*lock)
		);

		self.device
			.reset_descriptor_pool(*lock, vk::DescriptorPoolResetFlags::empty())
			.unwrap();
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasSynchronizedHandle<vk::DescriptorPool>, Deref, Borrow, Eq, Hash, Ord for DescriptorPool {
		target = { pool }
	}
}
impl Drop for DescriptorPool {
	fn drop(&mut self) {
		let lock = self.pool.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

		unsafe {
			self.device
				.destroy_descriptor_pool(*lock, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for DescriptorPool {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DescriptorPool")
			.field("device", &self.device)
			.field("pool", &self.pool)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
