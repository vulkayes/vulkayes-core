use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::{
	version::{DeviceV1_0, DeviceV1_1},
	vk::{self, DeviceQueueCreateFlags, DeviceQueueInfo2}
};

use crate::{device::Device, prelude::Vrc, sync::fence::Fence, util::sync::Vutex};

#[macro_use]
pub mod macros;

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
	const_queue_submit! {
		/// Example submit function generated using the [const_queue_submit!](../macro.const_queue_submit.html) macro.
		///
		/// At some point in the distant future this function will become const generic and the macro will be an implementation detail.
		pub fn submit_one(
			&queue,
			waits: [&Semaphore; 1],
			stages: [vk::PipelineStageFlags; _],
			buffers: [&CommandBuffer; 1],
			signals: [&Semaphore; 1],
			fence: Option<&Fence>
		) -> Result<(), QueueSubmitError>;
	}

	const_queue_present! {
		/// Example present function generated using the [const_queue_present!](../macro.const_queue_present.html) macro.
		///
		/// At some point in the distant future this function will become const generic and the macro will be an implementation detail.
		pub fn present_one(
			&queue,
			waits: [&Semaphore; 1],
			images: [&SwapchainImage; 1],
			result_for_all: bool
		) -> QueuePresentMultipleResult<[QueuePresentResult; _]>;
	}

	/// Gets a queue from the logical device.
	///
	/// ### Safety
	///
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetDeviceQueue.html>.
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetDeviceQueue2.html>.
	pub unsafe fn from_device(
		device: Vrc<Device>,
		flags: DeviceQueueCreateFlags,
		queue_family_index: u32,
		queue_index: u32
	) -> Vrc<Self> {
		log_trace_common!(
			"Creating queue:",
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
	/// This function will panic if the `Vutex` is poisoned.
	pub unsafe fn submit(
		&self,
		infos: impl AsRef<[vk::SubmitInfo]>,
		fence: Option<&Fence>
	) -> Result<(), error::QueueSubmitError> {
		let lock = self.queue.lock().expect("vutex poisoned");

		log_trace_common!(
			"Submitting on queue:",
			self,
			crate::util::fmt::format_handle(*lock),
			infos.as_ref(),
			fence
		);

		if let Some(fence) = fence {
			let fence_lock = fence.lock().expect("vutex poisoned");
			self.device
				.queue_submit(*lock, infos.as_ref(), *fence_lock)
				.map_err(Into::into)
		} else {
			self.device
				.queue_submit(*lock, infos.as_ref(), vk::Fence::null())
				.map_err(Into::into)
		}
	}

	/// Waits until all outstanding operations on the queue are completed.
	///
	/// ### Panic
	///
	/// This function will panic if the `Vutex` is poisoned.
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
	impl HasSynchronizedHandle<vk::Queue>, Deref, Borrow, Eq, Hash, Ord for Queue {
		target = { queue }
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
