//! This crate provides core components for the vulkayes project.
//!
//! ## Crate features:
//!
//! ### `host_allocator` and `rust_host_allocator`
//!
//! `host_allocator` adds `Custom` variant to `HostMemoryAllocator`. This makes the type sized, but enables the use of custom host memory allocators.
//!
//! `rust_host_allocator` adds `Rust()` constructor to `HostMemoryAllocator` that uses Rusts `std::alloc` methods. Requires `host_allocator` feature.
//!
//! ### `naive_device_allocator`
//!
//! Adds a simple memory allocator `NaiveDeviceMemoryAllocator` that allocates memory for each resource separately.
//! It should not be used in production applications.
//!
//! ### `multi_thread`
//!
//! Enables multi thread support by using `Arc<T>` and `Mutex<T>` (dubbed as `Vrc` and `Vutex`) instead of `Rc<T>` and `RefCell<T>` (wrapped to have compatible API).
//!
//! ### `parking_lot_vutex`
//!
//! Uses `Mutex` from `parking_lot` crate instead of the standard library. Requires `multi_thread` feature.
//!
//! ### `insecure_hash`
//!
//! Uses `rustc_hash::{FxHashMap, FxHashSet}` instead of `std::collections::{HashMap, HashSet}` (dubbed as `VHashMap` and `VHashSet`).
//!
//! ### `runtime_implicit_validations`
//!
//! Some implicit validations cannot be checked statically. This feature enables runtime checks of those validations.
//! Note that in some circumstances, such as Instance creation and extension name checking, the validation is part of the input
//! argument transformation and turning it off would not bring any advantages.
//!
//! These validations might not be cheap. It is recommended to only enabled them when debugging, not in release/production builds.
//!
//! ### `vulkan1_1` and `vulkan1_2`
//!
//! `vulkan1_1` enables methods that will panic on Vulkan 1.0
//!
//! `vulkan1_2` enables methods that will panic on Vulkan 1.0 and 1.1. Requires `vulkan1_1` feature.
//!
//! ### `log_max_level_*` and `log_release_max_level_*`
//!
//! These features directly correspond to the features on the `log` crate.

// Export `ash` because all other component will use it.
pub use ash;
// Export `log` so that `log_*` features can be applied to all vulkayes crates
pub use log;
// Export `seq_macro` because `lock_and_deref` macro requires it.
pub use seq_macro;

/// Non zero `1u32` constant to avoid unnecessary unsafe blocks in constant contexts.
pub const NONZEROU32_ONE: std::num::NonZeroU32 = unsafe { std::num::NonZeroU32::new_unchecked(1) };

// Macros used inside and outside of the crate.
#[macro_use]
pub mod util;

pub mod command;
pub mod descriptor;
pub mod device;
pub mod entry;
pub mod framebuffer;
pub mod instance;
pub mod memory;
pub mod physical_device;
pub mod pipeline;
pub mod prelude;
pub mod queue;
pub mod render_pass;
pub mod resource;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod sync;

#[cfg(test)]
mod test {
	pub fn setup_testing_logger() {
		static LOGGER_INITIALIZED: std::sync::atomic::AtomicBool =
			std::sync::atomic::AtomicBool::new(false);

		if LOGGER_INITIALIZED.compare_exchange(
			false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst
		).is_err() {
			let logger = edwardium_logger::Logger::new(
				[edwardium_logger::targets::stderr::StderrTarget::new(
					log::Level::Trace
				)],
				std::time::Instant::now()
			);
			logger.init_boxed().expect("Could not initialize logger");
		}
	}

	#[test]
	// Debug test for testing small thing during development
	fn debug() {
		setup_testing_logger();

		fn print_size<T>(name: &str) {
			log::info!(
				"{} size: {} align: {}",
				name,
				std::mem::size_of::<T>(),
				std::mem::align_of::<T>()
			);
		}

		print_size::<crate::memory::host::HostMemoryAllocator>("HostMemoryAllocator");

		print_size::<ash::Instance>("ash::Instance");
		print_size::<ash::Device>("ash::Instance");

		print_size::<crate::instance::Instance>("Instance");
		print_size::<crate::device::Device>("Device");
	}
}
