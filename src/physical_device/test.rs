#[test]
fn list_physical_devices() {
	crate::test::setup_testing_logger();

	const TEST_FORMAT: ash::vk::Format = ash::vk::Format::R8G8B8A8_SRGB;

	let instance = crate::instance::test::create_test_instance();
	for physical_device in instance.physical_devices().expect("Could not list physical devices") {
		let extensions_properties: Vec<_> =
			physical_device.extensions_properties().unwrap().collect();
		let format_properties = physical_device.physical_device_format_properties(TEST_FORMAT);
		let image_properties = physical_device
			.physical_device_image_format_properties(
				TEST_FORMAT,
				ash::vk::ImageType::TYPE_2D,
				ash::vk::ImageTiling::OPTIMAL,
				ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
				ash::vk::ImageCreateFlags::empty()
			)
			.expect(&format!("Format {:?} not supported", TEST_FORMAT));
		let memory_properties = physical_device.physical_device_memory_properties();
		let device_properties = physical_device.physical_device_properties();
		let queue_family_properties = physical_device.physical_device_queue_family_properties();
		let device_features = physical_device.physical_device_features();

		log::debug!(
			"{:?}: {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
			physical_device,
			extensions_properties,
			format_properties,
			image_properties,
			memory_properties,
			device_properties,
			queue_family_properties,
			device_features
		);
	}
}
