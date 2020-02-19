//! This crate provides core components for the vulkayes project.

// Export `ash` because all other component will use it.
pub use ash;
// Export `thiserror` because of the `vk_result_error` macro.
pub use thiserror;

// Export these fast hash-collections for other components to use.
// I found these to be fastest of { hashbrown, stdlib, fnv, fx } with a local benchmark, hopefully that's true for mostly everyone.
pub type FastHashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type FastHashSet<V> = rustc_hash::FxHashSet<V>;

#[cfg(not(feature = "non_atomic_vrc"))]
pub type Vrc<T> = std::sync::Arc<T>;
#[cfg(feature = "non_atomic_vrc")]
pub type Vrc<T> = std::rc::Rc<T>;

// Macros used inside and outside of the crate.
#[macro_use]
pub mod util;

pub mod memory;

pub mod device;
pub mod entry;
pub mod instance;
pub mod physical_device;

#[cfg(test)]
mod tests {
	use crate::memory::host::HostMemoryAllocator;

	use super::*;
	use crate::{instance::Instance, util::fmt::VkVersion};

	pub fn setup_testing_logger() {
		let logger = edwardium_logger::Logger::new(
			[edwardium_logger::targets::stderr::StderrTarget::new(log::Level::Trace)],
			std::time::Instant::now()
		);
		match logger.init_boxed() {
			Ok(_) => (),
			Err(_) => ()
		} // Purposely ignore the result as only the first test will set the logger successfully.
	}

	#[test]
	fn enumerate_layers_and_extensions() {
		setup_testing_logger();

		let entry = entry::Entry::new().expect("Could not create entry");

		entry.instance_layers().unwrap().for_each(|layer| {
			log::info!("Layer {}", layer);
		});

		entry.instance_extensions().unwrap().for_each(|extension| {
			log::info!("Extension {}", extension);
		});
	}

	#[cfg(feature = "rust_host_allocator")]
	#[test]
	fn create_instance_rust_host_allocator() {
		setup_testing_logger();

		instance::Instance::new(
			entry::Entry::new().unwrap(),
			instance::ApplicationInfo {
				application_name: "test",
				application_version: VkVersion::new(0, 1, 0),
				engine_name: "test",
				engine_version: VkVersion::new(0, 1, 0),
				api_version: ash::vk_make_version!(1, 2, 0).into()
			},
			None,
			None,
			HostMemoryAllocator::Rust(),
			instance::debug::DebugCallback::None()
		)
		.unwrap();
	}

	fn create_test_instance() -> Vrc<Instance> {
		instance::Instance::new(
			entry::Entry::new().unwrap(),
			instance::ApplicationInfo {
				engine_name: "test",
				api_version: ash::vk_make_version!(1, 2, 0).into(),
				..Default::default()
			},
			["VK_LAYER_LUNARG_standard_validation", "VK_LAYER_KHRONOS_validation"]
				.iter()
				.map(|&s| s),
			["VK_EXT_debug_report", "VK_EXT_debug_utils"].iter().map(|&s| s),
			HostMemoryAllocator::Unspecified(),
			instance::debug::DebugCallback::None()
		)
		.expect("Could not create instance")
	}

	#[test]
	fn list_physical_devices() {
		setup_testing_logger();

		const TEST_FORMAT: ash::vk::Format = ash::vk::Format::R8G8B8A8_SRGB;

		let instance = create_test_instance();
		for physical_device in instance.physical_devices().expect("Could not list physical devices")
		{
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
}
