//! This crate provides core components for the vulkayes project.

// Export `ash` because all other component will use it.
pub use ash;

// Export these fast hash-collections for other components to use.
// I found these to be fastest of { hashbrown, stdlib, fnv, fx } with a local benchmark, hopefully that's true for mostly everyone.
pub type FastHashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type FastHashSet<V> = rustc_hash::FxHashSet<V>;

// Macros used inside and outside of the crate.
#[macro_use]
pub mod util;

pub mod memory;

pub mod entry;
pub mod instance;
pub mod physical_device;
pub mod device;

#[cfg(test)]
mod tests {
	use crate::memory::host::HostMemoryAllocator;

	use super::*;

	pub fn setup_testing_logger() {
		let logger = edwardium_logger::Logger::new(
			[
				edwardium_logger::targets::stderr::StderrTarget::new(log::Level::Trace)
			],
			std::time::Instant::now()
		);
		logger.init_boxed();
	}

	#[test]
	fn enumerate_layers_and_extensions() {
		setup_testing_logger();

		let entry = entry::Entry::new().unwrap();

		entry.instance_layers().unwrap().for_each(|layer| {
			log::info!("Layer {}", layer);
		});

		entry.instance_extensions().unwrap().for_each(|extension| {
			log::info!("Extension {}", extension);
		});
	}

	#[cfg(feature = "rust_host_allocator")]
	#[test]
	fn create_instance() {
		setup_testing_logger();

		instance::Instance::new(
			entry::Entry::new().unwrap(),
			instance::ApplicationInfo {
				application_name: "test",
				application_version: 0,
				engine_name: "test",
				engine_version: 0,
				api_version: ash::vk_make_version!(1, 2, 0),
			},
			None,
			None,
			HostMemoryAllocator::Rust(),
			instance::debug::DebugCallback::None(),
		).unwrap();
	}
}
