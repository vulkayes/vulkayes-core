//! This crate provides core components for the vulkayes project.
//!
//! ## Crate features:
//!
//! ### `rust_host_allocator`
//!
//! Adds `Rust()` constructor to `HostMemoryAllocator` that uses Rusts `std::alloc` methods.
//!
//! ### `naive_device_allocator`
//!
//! Adds a simple memory allocator `NaiveDeviceMemoryAllocator` that allocates memory for each resource separately.
//! It should not be used in production applications.
//!
//! ### `single_thread`
//!
//! Replaces uses of `Arc<T>` and `Mutex<T>` (dubbed as `Vrc` and `Vutex` across the crate) with `Rc<T>` and `RefCell<T>` (wrapped to have compatible API).
//!
//! ### `crypto_secure_hash`
//!
//! Uses `std::collections::{HashMap, HashSet}` instead of `rustc_hash::{FxHashMap, FxHashSet}` (dubbed as `VHashMap` and `VHashSet`)  across the crate.
//!
//! ### `runtime_implicit_validations`
//!
//! Some implicit validations cannot be checked statically. This feature enables runtime checks of those validations.
//! Note that in some circumstances, such as Instance creation and extension name checking, the validation is part of the input
//! argument transformation and turning it off would not bring any advantages.

// Export `ash` because all other component will use it.
pub use ash;
// Export `seq_macro` because `lock_and_deref` macro from `queue` requires it.
pub use seq_macro;

pub use util::sync::Vrc;

/// Non zero `1u32` constant to avoid unnecessary unsafe blocks in constant contexts.
pub const NONZEROU32_ONE: std::num::NonZeroU32 = unsafe { std::num::NonZeroU32::new_unchecked(1) };

// Macros used inside and outside of the crate.
#[macro_use]
pub mod util;

pub mod memory;

pub mod command;
pub mod device;
pub mod entry;
pub mod instance;
pub mod physical_device;
pub mod queue;
pub mod resource;
pub mod surface;
pub mod swapchain;
pub mod sync;

#[cfg(test)]
mod test {
	pub fn setup_testing_logger() {
		static LOGGER_INITIALIZED: std::sync::atomic::AtomicBool =
			std::sync::atomic::AtomicBool::new(false);

		if LOGGER_INITIALIZED.compare_and_swap(false, true, std::sync::atomic::Ordering::SeqCst)
			== false
		{
			let logger = edwardium_logger::Logger::new(
				[edwardium_logger::targets::stderr::StderrTarget::new(
					log::Level::Trace
				)],
				std::time::Instant::now()
			);
			logger.init_boxed().expect("Could not initialize logger");
		}
	}
}
