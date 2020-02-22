use ash::vk::PhysicalDeviceFeatures;

use crate::{
	device::{Device, QueueCreateInfo},
	instance::Instance,
	queue::Queue,
	Vrc
};

#[test]
fn create_device() {
	crate::test::setup_testing_logger();

	let instance = crate::instance::test::create_test_instance();

	let queue_create_infos = [QueueCreateInfo {
		queue_family_index: 0,
		flags: Default::default(),
		queue_priorities: [1.0]
	}];

	for physical_device in instance.physical_devices().unwrap() {
		let _device = Device::new(
			physical_device,
			&queue_create_infos,
			["VK_LAYER_LUNARG_standard_validation", "VK_LAYER_KHRONOS_validation"]
				.iter()
				.map(|&s| s),
			None,
			Default::default(),
			Default::default()
		)
		.expect("Could not create device");
	}
}

pub fn create_test_device(
	instance: Vrc<Instance>, index: usize, features: PhysicalDeviceFeatures
) -> (Vrc<Device>, Vec<Vrc<Queue>>) {
	let physical_device = instance
		.physical_devices()
		.unwrap()
		.nth(index)
		.expect(&format!("Could not fine physical device with index {}", index));
	Device::new(
		physical_device,
		[QueueCreateInfo {
			queue_family_index: 0,
			flags: Default::default(),
			queue_priorities: [1.0]
		}],
		["VK_LAYER_LUNARG_standard_validation", "VK_LAYER_KHRONOS_validation"].iter().map(|&s| s),
		None,
		features,
		Default::default()
	)
	.expect("Could not create test device")
}
