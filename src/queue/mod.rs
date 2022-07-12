use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::vk::{self, DeviceQueueCreateFlags, DeviceQueueInfo2};

use crate::prelude::{CommandBuffer, Device, Fence, Semaphore, SwapchainImage, Vrc, Vutex};

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
	pub fn submit<const WAITS: usize, const BUFFERS: usize, const SIGNALS: usize>(
		&self,
		wait_for: [&Semaphore; WAITS],
		wait_for_stages: [vk::PipelineStageFlags; WAITS],
		buffers: [&CommandBuffer; BUFFERS],
		signal_after: [&Semaphore; SIGNALS],
		fence: Option<&Fence>
	) -> Result<(), error::QueueSubmitError> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			for stage in wait_for_stages.iter() {
				if stage.is_empty() {
					return Err(error::QueueSubmitError::WaitStagesEmpty)
				}
			}
			{
				// check that all waits, buffers and signals come from the same device
				if !crate::util::validations::validate_all_match(
					wait_for
						.iter()
						.map(|w| w.device())
						.chain(buffers.iter().map(|b| b.pool().device()))
						.chain(signal_after.iter().map(|s| s.device()))
				) {
					return Err(error::QueueSubmitError::WaitBufferSignalDeviceMismatch)
				}
			}
			for cb in buffers.iter() {
				if cb.pool().queue_family_index() != self.queue_family_index() {
					return Err(error::QueueSubmitError::QueueFamilyMismatch)
				}
			}
			if let Some(ref fence) = fence {
				if self.device() != fence.device() {
					return Err(error::QueueSubmitError::QueueFenceDeviceMismatch)
				}
			}
		}

		let wait_for_locks = wait_for.map(|s| s.lock().expect("vutex poisoned"));
		let wait_for_raw = wait_for_locks.map(|l| *l);

		let buffers_locks = buffers.map(|s| s.lock().expect("vutex poisoned"));
		let buffers_raw = buffers_locks.map(|l| *l);

		let signal_after_locks = signal_after.map(|s| s.lock().expect("vutex poisoned"));
		let signal_after_raw = signal_after_locks.map(|l| *l);

		let submit_info = vk::SubmitInfo::builder()
			.wait_semaphores(&wait_for_raw)
			.wait_dst_stage_mask(&wait_for_stages)
			.command_buffers(&buffers_raw)
			.signal_semaphores(&signal_after_raw)
			.build();

		unsafe { self.submit_raw([submit_info], fence) }
	}

	pub fn present_with_all_results<const WAITS: usize, const IMAGES: usize>(
		&self,
		wait_for: [&Semaphore; WAITS],
		images: [&SwapchainImage; IMAGES]
	) -> [Result<error::QueuePresentSuccess, error::QueuePresentError>; IMAGES] {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if IMAGES == 0 {
				return [(); IMAGES].map(|_| Err(error::QueuePresentError::SwapchainsEmpty))
			}
			if !crate::util::validations::validate_all_match(
				images
					.iter()
					.map(|&i| i.device().instance())
					.chain(wait_for.iter().map(|&w| w.device().instance()))
			) {
				return [(); IMAGES].map(|_| Err(error::QueuePresentError::SwapchainsSempahoredInstanceMismatch))
			}
		}

		let any_swapchain = images[0].swapchain();

		let wait_for_locks = wait_for.map(|s| s.lock().expect("vutex poisoned"));
		let wait_for_raw = wait_for_locks.map(|l| *l);

		let swapchains_locks = images.map(|i| i.swapchain().lock().expect("vutex poisoned"));
		let swapchains_raw = swapchains_locks.map(|l| *l);

		let indices = images.map(|i| i.index());

		let mut results = [vk::Result::SUCCESS; IMAGES];

		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&wait_for_raw)
			.swapchains(&swapchains_raw)
			.image_indices(&indices)
			.results(&mut results);

		let _ = unsafe { any_swapchain.present(self, present_info) };

		results.map(error::match_queue_present_result)
	}

	pub fn present<const WAITS: usize, const IMAGES: usize>(
		&self,
		wait_for: [&Semaphore; WAITS],
		images: [&SwapchainImage; IMAGES]
	) -> Result<error::QueuePresentSuccess, error::QueuePresentError> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if IMAGES == 0 {
				return Err(error::QueuePresentError::SwapchainsEmpty)
			}
			if !crate::util::validations::validate_all_match(
				images
					.iter()
					.map(|&i| i.device().instance())
					.chain(wait_for.iter().map(|&w| w.device().instance()))
			) {
				return Err(error::QueuePresentError::SwapchainsSempahoredInstanceMismatch)
			}
		}

		let any_swapchain = images[0].swapchain();

		let wait_for_locks = wait_for.map(|s| s.lock().expect("vutex poisoned"));
		let wait_for_raw = wait_for_locks.map(|l| *l);

		let swapchains_locks = images.map(|i| i.swapchain().lock().expect("vutex poisoned"));
		let swapchains_raw = swapchains_locks.map(|l| *l);

		let indices = images.map(|i| i.index());

		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&wait_for_raw)
			.swapchains(&swapchains_raw)
			.image_indices(&indices);

		unsafe { any_swapchain.present(self, present_info) }
	}

	/// Gets a queue from the logical device.
	///
	/// ### Safety
	///
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetDeviceQueue.html>.
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetDeviceQueue2.html>.
	pub unsafe fn from_device(device: Vrc<Device>, flags: DeviceQueueCreateFlags, queue_family_index: u32, queue_index: u32) -> Vrc<Self> {
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
			device.fp_v1_1().get_device_queue2(
				device.handle(),
				info.deref(),
				mem.as_mut_ptr()
			);

			mem.assume_init()
		};

		Vrc::new(Queue { device, queue: Vutex::new(queue), queue_family_index, queue_index })
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
	pub unsafe fn submit_raw(&self, infos: impl AsRef<[vk::SubmitInfo]>, fence: Option<&Fence>) -> Result<(), error::QueueSubmitError> {
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
			.field(
				"queue_family_index",
				&self.queue_family_index
			)
			.field("queue_index", &self.queue_index)
			.finish()
	}
}
