use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::{
	version::{DeviceV1_0, DeviceV1_1},
	vk::{self, DeviceQueueCreateFlags, DeviceQueueInfo2}
};

use crate::{device::Device, util::sync::Vutex, Vrc};
use crate::sync::fence::Fence;

pub mod error;
pub mod sharing_mode;

/// Creates two fixed-size arrays. The first one holds locks and the second one holds deref of those locks.
///
/// Usage:
/// ```
/// lock_and_deref!(let foo[2]{.lock().unwrap()} => foo_locks: [LockGuard<Foo>; 2] => foo_derefs;)
/// ```
/// expands to
/// ```
/// let foo_locks: [LockGuard<Foo>; 2] = [foo[0].lock().unwrap(), foo[1].lock().unwrap()];
/// let foo_derefs = [*foo_locks[0], *foo_locks[1]];
/// ```
///
/// This macro uses a `proc-macro-hack` version of the `seq-macro` crate to generate the array indices.
#[macro_export]
macro_rules! lock_and_deref {
	(
		let $ex: ident[$count: literal] {$($lock_code: tt)+} => $locks: ident $(: $l_type: ty)? => $derefs: ident;
	) => {
		#[allow(unused_variables)]
		let $locks $(: $l_type)? = $crate::seq_macro::seq_expr!(
			N in 0 .. $count {
				[
					#( $ex[N] $($lock_code)+, )*
				]
			}
		);
		#[allow(unused_variables)]
		let $derefs = $crate::seq_macro::seq_expr!(
			N in 0 .. $count {
				[
					#( *$locks[N], )*
				]
			}
		);
	}
}
/// This macro is intended to substitute for const generics when transforming input arguments to the `queue.submit` function.
///
/// Usage:
/// ```
/// const_queue_submit! {
/// 	pub fn submit_one(
/// 		&queue,
/// 		waits: [_; 1],
/// 		stages,
/// 		buffers: [_; 1],
/// 		signals: [_; 1],
/// 		fence
/// 	) -> Result<(), QueueSubmitError>;
/// }
/// ```
///
/// this expands to something like the [queue.submit_one](queue/struct.Queue.html#method.submit_one)
#[macro_export]
macro_rules! const_queue_submit {
	(
		$(#[$attribute: meta])*
		pub fn $name: ident (
			&queue,
			$waits: ident: [_; $count_waits: literal],
			stages,
			$buffers: ident: [_; $count_buffers: literal],
			$signals: ident: [_; $count_signals: literal],
			fence
		) -> Result<(), QueueSubmitError>;
	) => {
		$(#[$attribute])*
		#[allow(unused_variables)]
		pub fn $name(
			queue: &$crate::queue::Queue,
			$waits: [&$crate::sync::semaphore::Semaphore; $count_waits],
			stages: [$crate::ash::vk::PipelineStageFlags; $count_waits],
			$buffers: [&$crate::command::buffer::CommandBuffer; $count_buffers],
			$signals: [&$crate::sync::semaphore::Semaphore; $count_signals],
			fence: Option<&$crate::sync::fence::Fence>
		) -> Result<(), $crate::queue::error::QueueSubmitError> {
			if cfg!(feature = "runtime_implicit_validations") {
				for stage in stages.iter() {
					if stage.is_empty() {
						return Err($crate::queue::error::QueueSubmitError::WaitStagesEmpty)
					}
				}
				{ // check that all waits, buffers and signals come from the same device
					match $waits.iter().map(|w| w.device()).chain(
						$buffers.iter().map(|b| b.device())
					).chain(
						$signals.iter().map(|s| s.device())
					).try_fold(None, |acc, d| {
						match acc {
							None => Ok(Some(d)),
							Some(common) => {
								if common == d {
									Ok(Some(common))
								} else {
									Err(())
								}
							}
						}
					}) {
						Err(_) => return Err($crate::queue::error::QueueSubmitError::WaitBufferSignalDeviceMismatch),
						_ => ()
					}
				}
				for cb in $buffers.iter() {
					if cb.pool().queue_family_index() != queue.queue_family_index() {
						return Err($crate::queue::error::QueueSubmitError::QueueFamilyMismatch)
					}
				}
				if let Some(ref fence) = fence {
					if queue.device() != fence.device() {
						return Err($crate::queue::error::QueueSubmitError::QueueFenceDeviceMismatch)
					}
				}
			}

			$crate::lock_and_deref!(
				let $waits[$count_waits]{.lock().expect("vutex poisoned")} => $waits: [$crate::util::sync::VutexGuard<$crate::ash::vk::Semaphore>; $count_waits] => w;
			);
			$crate::lock_and_deref!(
				let $buffers[$count_buffers]{.lock().expect("vutex poisoned")} => $buffers: [$crate::util::sync::VutexGuard<$crate::ash::vk::CommandBuffer>; $count_buffers] => b;
			);
			$crate::lock_and_deref!(
				let $signals[$count_signals]{.lock().expect("vutex poisoned")} => $signals: [$crate::util::sync::VutexGuard<$crate::ash::vk::Semaphore>; $count_signals] => s;
			);

			let submit_info = $crate::ash::vk::SubmitInfo::builder()
				.wait_semaphores(&w)
				.wait_dst_stage_mask(&stages)
				.command_buffers(&b)
				.signal_semaphores(&s)
				.build()
			;

			unsafe {
				queue.submit(
					[submit_info],
					fence
				)
			}
		}
	}
}

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

	const_queue_submit! {
		/// Example submit function generated using the `const_queue_submit` macro.
		///
		/// At some point in the distant future this function will become const generic and the macro will be an implementation detail.
		pub fn submit_one(
			&queue,
			waits: [_; 1],
			stages,
			buffers: [_; 1],
			signals: [_; 1],
			fence
		) -> Result<(), QueueSubmitError>;
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
	pub unsafe fn submit<F: Deref<Target = Fence>>(
		&self,
		infos: impl AsRef<[vk::SubmitInfo]>,
		fence: Option<F>
	) -> Result<(), error::QueueSubmitError> {
		let lock = self.queue.lock().expect("vutex poisoned");

		log_trace_common!(
			"Submitting on queue:",
			crate::util::fmt::format_handle(*lock),
			infos.as_ref(),
			fence.as_deref()
		);

		if let Some(fence) = fence {
			let fence_lock = fence.lock().expect("vutex poisoned");
			self.device.queue_submit(
				*lock,
				infos.as_ref(),
				*fence_lock
			).map_err(Into::into)
		} else {
			self.device.queue_submit(
				*lock,
				infos.as_ref(),
				vk::Fence::null()
			).map_err(Into::into)
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
