//! Utilities and macros.

#[macro_use]
pub mod macros;

#[macro_use]
pub mod fmt;

#[macro_use]
pub mod sync;

pub mod hash;
pub mod string;
pub mod validations;
pub mod handle;
pub mod transparent;

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

/// `align_up(base, align)` returns the smallest greater integer than `base` aligned to `align`.
///
/// More formally:
/// ```text
/// f_d(x) =
///     x, if x mod d = 0
///     x + d - x mod d, otherwise
/// ```
///
/// simplifies to `x - 1 + d - (x - 1) mod d`
/// assuming `d = 2^N`, can also be written in code like: `(x - 1 + d) & !(d - 1)`
/// where `x = base` and `d = align`
///
/// Similar code to `std::alloc::Layout::padding_needed_for`, but without the `- base`
pub const fn align_up(base: usize, align: usize) -> usize {
	base.wrapping_add(align.wrapping_sub(1)) & !align.wrapping_sub(1)
}
