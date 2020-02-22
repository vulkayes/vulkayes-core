use crate::{
	entry,
	instance,
	instance::Instance,
	memory::host::HostMemoryAllocator,
	util::fmt::VkVersion,
	Vrc
};

#[test]
fn create_instance() {
	crate::test::setup_testing_logger();

	instance::Instance::new(
		entry::Entry::new().unwrap(),
		Default::default(),
		None,
		None,
		HostMemoryAllocator::Unspecified(),
		instance::debug::DebugCallback::None()
	)
	.unwrap();
}

#[cfg(feature = "rust_host_allocator")]
#[test]
fn create_instance_rust_host_allocator() {
	crate::test::setup_testing_logger();

	instance::Instance::new(
		entry::Entry::new().unwrap(),
		instance::ApplicationInfo {
			application_name: "test",
			application_version: VkVersion::new(0, 1, 0),
			engine_name: "test",
			engine_version: VkVersion::new(0, 1, 0),
			api_version: VkVersion::new(1, 2, 0)
		},
		None,
		None,
		HostMemoryAllocator::Rust(),
		instance::debug::DebugCallback::None()
	)
	.unwrap();
}

pub fn create_test_instance() -> Vrc<Instance> {
	instance::Instance::new(
		entry::Entry::new().unwrap(),
		instance::ApplicationInfo {
			engine_name: "test",
			api_version: VkVersion::new(1, 2, 0),
			..Default::default()
		},
		["VK_LAYER_LUNARG_standard_validation", "VK_LAYER_KHRONOS_validation"].iter().map(|&s| s),
		["VK_EXT_debug_report", "VK_EXT_debug_utils"].iter().map(|&s| s),
		HostMemoryAllocator::Unspecified(),
		instance::debug::DebugCallback::None()
	)
	.expect("Could not create instance")
}
