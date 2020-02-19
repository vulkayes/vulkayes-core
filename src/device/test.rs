use crate::{
	device::{Device, QueueCreateInfo},
	instance::Instance,
	Vrc
};
use ash::vk::PhysicalDeviceFeatures;

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
			instance.clone(),
			&queue_create_infos,
			["VK_LAYER_LUNARG_standard_validation", "VK_LAYER_KHRONOS_validation"]
				.iter()
				.map(|&s| s),
			["VK_KHR_swapchain"].iter().map(|&s| s),
			Default::default(),
			physical_device,
			Default::default()
		)
		.expect("Could not create device");
	}
}

pub fn create_test_device(
	instance: Vrc<Instance>, index: usize, features: PhysicalDeviceFeatures
) -> Vrc<Device> {
	let physical_device = instance
		.physical_devices()
		.unwrap()
		.nth(index)
		.expect(&format!("Could not fine physical device with index {}", index));
	Device::new(
		instance,
		[QueueCreateInfo {
			queue_family_index: 0,
			flags: Default::default(),
			queue_priorities: [1.0]
		}],
		["VK_LAYER_LUNARG_standard_validation", "VK_LAYER_KHRONOS_validation"].iter().map(|&s| s),
		None,
		features,
		physical_device,
		Default::default()
	)
	.expect("Could not create test device")
}
