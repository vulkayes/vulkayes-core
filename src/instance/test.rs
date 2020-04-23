use crate::{entry, instance, memory::host::HostMemoryAllocator, util::fmt::VkVersion};

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
