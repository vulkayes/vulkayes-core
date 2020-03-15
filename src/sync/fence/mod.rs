use std::{
	fmt::{self, Debug},
	ops::Deref
};

use ash::{version::DeviceV1_0, vk};

use crate::{device::Device, memory::host::HostMemoryAllocator, util::sync::Vutex, Vrc};

pub mod error;

#[derive(Debug, Copy, Clone)]
pub enum WaitTimeout {
	/// Don't wait, return immediately
	None,
	/// Specify a timeout in nanosecond
	Timeout(u64),
	/// Wait forever
	Forever
}
impl Into<u64> for WaitTimeout {
	fn into(self) -> u64 {
		match self {
			WaitTimeout::None => 0,
			WaitTimeout::Timeout(t) => t,
			WaitTimeout::Forever => std::u64::MAX
		}
	}
}
impl Default for WaitTimeout {
	fn default() -> Self {
		WaitTimeout::Forever
	}
}

pub struct Fence {
	device: Vrc<Device>,
	fence: Vutex<vk::Fence>,

	host_memory_allocator: HostMemoryAllocator
}
impl Fence {
	pub fn new(
		device: Vrc<Device>,
		signaled: bool,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::FenceError> {
		let flags = if signaled {
			vk::FenceCreateFlags::SIGNALED
		} else {
			vk::FenceCreateFlags::empty()
		};
		let create_info = vk::FenceCreateInfo::builder().flags(flags);

		unsafe { Self::from_create_info(device, create_info, host_memory_allocator) }
	}

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

		let fence = device.create_fence(create_info.deref(), host_memory_allocator.as_ref())?;

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
	pub fn wait(&self, timeout: WaitTimeout) -> Result<bool, error::FenceError> {
		let lock = self.fence.lock().expect("vutex poisoned");

		// Unfortunately this is an ash API design bug that it doesn't return bool from wait_for_fences
		let result = unsafe {
			self.device.fp_v1_0().wait_for_fences(
				self.device.deref().deref().handle(),
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
	impl Deref, PartialEq, Eq, Hash for Fence {
		type Target = Vutex<ash::vk::Fence> { fence }

		to_handle { .lock().expect("vutex poisoned").deref() }
	}
}
impl Drop for Fence {
	fn drop(&mut self) {
		let lock = self.fence.lock().expect("vutex poisoned");

		unsafe {
			self.device
				.destroy_fence(*lock, self.host_memory_allocator.as_ref())
		}
	}
}
impl Debug for Fence {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Fence")
			.field("device", &self.device)
			.field("Fence", &self.fence)
			.field("allocation_callbacks", &self.host_memory_allocator)
			.finish()
	}
}
