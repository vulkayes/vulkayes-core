use std::{
	fmt::{self, Debug},
	ops::Deref
};

use ash::vk;

use crate::{device::Device, memory::host::HostMemoryAllocator, prelude::Vrc, util::sync::Vutex};

pub mod error;

pub struct Fence {
	device: Vrc<Device>,
	fence: Vutex<vk::Fence>,

	host_memory_allocator: HostMemoryAllocator
}
impl Fence {
	pub fn new(device: Vrc<Device>, signaled: bool, host_memory_allocator: HostMemoryAllocator) -> Result<Vrc<Self>, error::FenceError> {
		let flags = if signaled { vk::FenceCreateFlags::SIGNALED } else { vk::FenceCreateFlags::empty() };
		let create_info = vk::FenceCreateInfo::builder().flags(flags);

		unsafe {
			Self::from_create_info(
				device,
				create_info,
				host_memory_allocator
			)
		}
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateFence.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::FenceCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::FenceError> {
		log_trace_common!(
			"Creating fence:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let fence = device.create_fence(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(Fence {
			device,
			fence: Vutex::new(fence),
			host_memory_allocator
		}))
	}

	/// Returns status of the fence where `true` means signalled and `false` means unsignaled.
	///
	/// ### Panic
	///
	/// This function will panic if the `Vutex` is poisoned.
	pub fn status(&self) -> Result<bool, error::FenceStatusError> {
		let lock = self.fence.lock().expect("vutex poisoned");

		unsafe { self.device.get_fence_status(*lock).map_err(Into::into) }
	}

	pub fn reset(&self) -> Result<(), error::FenceError> {
		let lock = self.fence.lock().expect("vutex poisoned");

		unsafe { self.device.reset_fences(&[*lock]).map_err(Into::into) }
	}

	/// Waits for `self` with an optional timeout.
	///
	/// Returns `false` if the timeout expires before the fence is signaled.
	pub fn wait(&self, timeout: crate::util::WaitTimeout) -> Result<bool, error::FenceError> {
		let lock = self.fence.lock().expect("vutex poisoned");

		// Unfortunately this is an ash API design bug that it doesn't return bool from wait_for_fences
		let result = unsafe {
			self.device.fp_v1_0().wait_for_fences(
				self.device.handle(),
				1u32,
				[*lock].as_ptr(),
				false as u32,
				timeout.into()
			)
		};

		match result {
			vk::Result::SUCCESS => Ok(true),
			vk::Result::TIMEOUT => Ok(false),
			_ => Err(result.into())
		}
	}

	// TODO: Specialcase `wait_any` and `wait_all` for exactly two fences for now?

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasSynchronizedHandle<vk::Fence>, Deref, Borrow, Eq, Hash, Ord for Fence {
		target = { fence }
	}
}
impl Drop for Fence {
	fn drop(&mut self) {
		let lock = self.fence.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

		unsafe {
			self.device.destroy_fence(
				*lock,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl Debug for Fence {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Fence")
			.field("device", &self.device)
			.field("Fence", &self.fence)
			.field(
				"allocation_callbacks",
				&self.host_memory_allocator
			)
			.finish()
	}
}
