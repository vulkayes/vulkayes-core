use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::{
	version::{DeviceV1_0, DeviceV1_1},
	vk::{DeviceQueueCreateFlags, DeviceQueueInfo2}
};

use crate::{device::Device, Vrc};

/// An internally synchronized device queue.
pub struct Queue {
	device: Vrc<Device>,
	queue: ash::vk::Queue,

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
		device: Vrc<Device>, flags: DeviceQueueCreateFlags, queue_family_index: u32,
		queue_index: u32
	) -> Self {
		let queue = if flags == DeviceQueueCreateFlags::empty() {
			device.get_device_queue(queue_family_index, queue_index)
		} else {
			let mut mem = std::mem::MaybeUninit::uninit();

			let info = DeviceQueueInfo2::builder()
				.flags(flags)
				.queue_family_index(queue_family_index)
				.queue_index(queue_index);
			device.fp_v1_1().get_device_queue2(device.handle(), info.deref(), mem.as_mut_ptr());

			mem.assume_init()
		};

		Queue { device, queue, queue_family_index, queue_index }
	}

	pub fn device(&self) -> &Vrc<Device> { &self.device }

	pub fn queue_family_index(&self) -> u32 { self.queue_family_index }

	pub fn queue_index(&self) -> u32 { self.queue_index }
}
impl Deref for Queue {
	type Target = ash::vk::Queue;

	fn deref(&self) -> &Self::Target { &self.queue }
}
impl Debug for Queue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Queue")
			.field("device", &self.device)
			.field("queue", &crate::util::fmt::format_handle(self.queue))
			.field("queue_family_index", &self.queue_family_index)
			.field("queue_index", &self.queue_index)
			.finish()
	}
}
