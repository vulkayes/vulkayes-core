use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::{
	version::{DeviceV1_0, DeviceV1_1},
	vk::{self, DeviceQueueCreateFlags, DeviceQueueInfo2}
};

use crate::{device::Device, util::sync::Vutex, Vrc};

pub mod error;
pub mod sharing_mode;

/// An internally synchronized device queue.
pub struct Queue {
	device: Vrc<Device>,
	queue: Vutex<ash::vk::Queue>,

	// TODO: Creation flags?
	queue_family_index: u32,
	queue_index: u32
}
impl Queue {
	/// Gets a queue from the logical device.
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetDeviceQueue.html>.
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetDeviceQueue2.html>.
	pub unsafe fn from_device(
		device: Vrc<Device>,
		flags: DeviceQueueCreateFlags,
		queue_family_index: u32,
		queue_index: u32
	) -> Vrc<Self> {
		log::trace!(
			"Creating queue with {:#?} {:#?} {:#?} {:#?}",
			device,
			flags,
			queue_family_index,
			queue_index
		);
		let queue = if flags.is_empty() {
			device.get_device_queue(queue_family_index, queue_index)
		} else {
			let mut mem = std::mem::MaybeUninit::uninit();

			let info = DeviceQueueInfo2::builder()
				.flags(flags)
				.queue_family_index(queue_family_index)
				.queue_index(queue_index);
			device
				.fp_v1_1()
				.get_device_queue2(device.handle(), info.deref(), mem.as_mut_ptr());

			mem.assume_init()
		};

		Vrc::new(Queue {
			device,
			queue: Vutex::new(queue),
			queue_family_index,
			queue_index
		})
	}

	/// Submits to given queue.
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkQueueSubmit.html>
	///
	/// ### Panic
	///
	/// This function will panic if the `Vutex` is posioned.
	pub unsafe fn submit(
		&self,
		infos: impl AsRef<[vk::SubmitInfo]>,
		fence: vk::Fence // TODO: Smart fence wrapper?
	) -> Result<(), error::QueueSubmitError> {
		let lock = self.queue.lock().expect("vutex poisoned");

		log::trace!(
			"Submitting on queue {:#?} {:#?} {:#?}",
			crate::util::fmt::format_handle(*lock),
			infos.as_ref(),
			fence
		);

		self.device
			.queue_submit(*lock, infos.as_ref(), fence)
			.map_err(Into::into)
	}

	/// Waits until all outstanding operations on the queue are completed.
	pub fn wait(&self) -> Result<(), error::QueueWaitError> {
		let lock = self.queue.lock().expect("vutex poisoned");

		unsafe { self.device.queue_wait_idle(*lock).map_err(Into::into) }
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub const fn queue_family_index(&self) -> u32 {
		self.queue_family_index
	}

	pub const fn queue_index(&self) -> u32 {
		self.queue_index
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for Queue {
		type Target = Vutex<ash::vk::Queue> { queue }

		to_handle { .lock().expect("vutex poisoned").deref() }
	}
}
impl Debug for Queue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Queue")
			.field("device", &self.device)
			.field("queue", &self.queue)
			.field("queue_family_index", &self.queue_family_index)
			.field("queue_index", &self.queue_index)
			.finish()
	}
}
