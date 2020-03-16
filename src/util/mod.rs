//! Utilities and macros.

#[macro_use]
pub mod macros;

#[macro_use]
pub mod fmt;
pub mod hash;
pub mod string;
pub mod sync;
pub mod validations;

#[derive(Debug, Copy, Clone)]
pub enum WaitTimeout {
	/// Don't wait, return immediately
	None,
	/// Specify a timeout in nanosecond
	Timeout(u64),
	/// Wait forever
	Forever
}
impl Into<u64> for WaitTimeout {
	fn into(self) -> u64 {
		match self {
			WaitTimeout::None => 0,
			WaitTimeout::Timeout(t) => t,
			WaitTimeout::Forever => std::u64::MAX
		}
	}
}
impl Default for WaitTimeout {
	fn default() -> Self {
		WaitTimeout::Forever
	}
}
