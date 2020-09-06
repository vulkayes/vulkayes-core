use std::fmt::{Debug, Display, Formatter, Result};

#[macro_export]
macro_rules! log_trace_common {
	(
		$title: literal,
		$(
			$log_item: expr
		),*
	) => {
		log_trace_common!(
			trace;
			$title,
			$(
				$log_item
			),*
		)
	};

	(
		$not_trace: ident;
		$title: literal,
		$(
			$log_item: expr
		),*
	) => {
		log::$not_trace!(
			concat!(
				$title,
				$(
					concat!("\n\t", stringify!($log_item), " = ", "{:?}")
				),*
			),
			$(
				$log_item
			),*
		)
	};
}


/// ```
/// # use vulkayes_core::debugize_struct;
/// # use std::ptr::null;
/// # #[derive(Debug)]
/// # struct TesterInnerFoo { foo: usize }
/// # struct TesterInnerBar { p_bar: *const usize }
/// # struct TesterInnerBaz { p_baz: *const usize, baz_size: u32 }
/// # struct Tester { foo: usize, p_bar: *const usize, p_baz: *const usize, baz_size: u32, foo_r: TesterInnerFoo, p_bar_r: *const TesterInnerBar, p_baz_r: *const TesterInnerBaz, baz_r_size: usize, n_qux: usize }
/// # let create_info = Tester { foo: 1, p_bar: null(), p_baz: null(), baz_size: 0, foo_r: TesterInnerFoo { foo: 2 }, p_bar_r: null(), p_baz_r: null(), baz_r_size: 0, n_qux: 3 };
///
/// let debuggable_value = unsafe {
/// 	debugize_struct!(
/// 		create_info;
/// 		{
/// 			// Simple alias
/// 			foo: foo; // => create_info.foo
/// 			// Depointerization
/// 			bar: *p_bar; // => create_info.p_bar.as_ref()
/// 			// Slice depointerization - unsafe
/// 			baz: *[baz_size] p_baz; // => create_info.p_baz.as_ref().map(|r| std::slice::from_raw_parts(r, create_info.baz_size))
/// 			// Recursion
/// 			foo_r: { foo: foo; } from foo_r;
/// 			bar_r: { bar: *p_bar; } from *p_bar_r;
/// 			baz_r: { baz: *[baz_size] p_baz; } from *[baz_r_size] p_baz_r;
/// 			// Custom closure
/// 			qux: n_qux | { n_qux + 1 }; // (|n_qux| { n_qux + 1 })(create_info.n_qux)
/// 		}
/// 	)
/// };
///
/// dbg!(debuggable_value);
/// ```
#[macro_export]
macro_rules! debugize_struct {
	(
		$input: expr;
		{
			$(
				$field: ident: $({ $($recursion: tt)+ } from)? $(* $([$size_target: ident])? )? $target: ident $(| { $($closure: tt)+ })?;
			)+
		}
	) => {
		{
			#[allow(non_camel_case_types)]
			struct Debugizer<$($field),+> {
				$($field: $field),+
			}
			#[allow(non_camel_case_types)]
			impl<$($field: std::fmt::Debug),+> std::fmt::Debug for Debugizer<$($field),+> {
				fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
					f.debug_struct("")
					$(
						.field(stringify!($field), &self.$field)
					)+
					.finish()
				}
			}

			Debugizer {
				$(
					$field: $crate::debugize_struct!(
						$input;
						: $({ $($recursion)+ } from)? $(* $([$size_target])? )? $target $(| { $($closure)+ } )?
					)
				),+
			}
		}
	};

	// Recursion
	(
		$input: expr;
		: { $($recursion: tt)+ } from $target: ident
	) => {
		$crate::debugize_struct!(
			$input.$target;
			{
				$($recursion)+
			}
		)
	};

	(
		$input: expr;
		: { $($recursion: tt)+ } from *$target: ident
	) => {
		$crate::debugize_struct!($input; : *$target).map(
			|r| {
				$crate::debugize_struct!(
					r;
					{
						$($recursion)+
					}
				)
			}
		)
	};

	(
		$input: expr;
		: { $($recursion: tt)+ } from *[$size_target: ident] $target: ident
	) => {
		$crate::debugize_struct!($input; : *[$size_target] $target).map(
			|s| s.iter().map(|e| {
					$crate::debugize_struct!(
						e;
						{
							$($recursion)+
						}
					)
				}
			).collect::<Vec<_>>()
		)
	};

	// Custom closure
	(
		$input: expr;
		: $target: ident | { $($closure: tt)+ }
	) => {
		(|$target| { $($closure)+ })($input.$target)
	};

	// Slice depointerization
	(
		$input: expr;
		: *[$size_target: ident] $target: ident
	) => {
		$input.$target.as_ref().map(|r| std::slice::from_raw_parts(r, $input.$size_target as usize))
	};

	// Depointerization
	(
		$input: expr;
		: *$target: ident
	) => {
		$input.$target.as_ref()
	};

	// Simple alias
	(
		$input: expr;
		: $target: ident
	) => {
		$input.$target
	};
}

pub fn log_vulkayes_debug_info() {
	log::debug!(
		"Enabled features:
	host_allocator: {}
	rust_host_allocator: {}
	naive_device_allocator: {}
	multi_thread: {}
	insecure_hash: {}
	runtime_implicit_validations: {}
	vulkan1_1: {}
	vulkan1_2: {}
",
		cfg!(feature = "host_allocator"),
		cfg!(feature = "rust_host_allocator"),
		cfg!(feature = "naive_device_allocator"),
		cfg!(feature = "multi_thread"),
		cfg!(feature = "insecure_hash"),
		cfg!(feature = "runtime_implicit_validations"),
		cfg!(feature = "vulkan1_1"),
		cfg!(feature = "vulkan1_2"),
	);
}

/// Formats Vulkan handle as `<ObjectType $raw>`.
pub fn format_handle<H: ash::vk::Handle>(handle: H) -> impl Debug + Display {
	struct Inner {
		ty: ash::vk::ObjectType,
		raw: u64
	}
	impl Debug for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result {
			write!(f, "<{:?} 0x{:x}>", self.ty, self.raw)
		}
	}
	impl Display for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result {
			write!(f, "<{:?} 0x{:x}>", self.ty, self.raw)
		}
	}

	Inner {
		ty: H::TYPE,
		raw: handle.as_raw()
	}
}

#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct VkVersion(pub u32);
impl VkVersion {
	pub fn new(major: u32, minor: u32, patch: u32) -> Self {
		VkVersion(ash::vk::make_version(major, minor, patch))
	}
}
impl Debug for VkVersion {
	fn fmt(&self, f: &mut Formatter) -> Result {
		<VkVersion as Display>::fmt(self, f)
	}
}
impl Display for VkVersion {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(
			f,
			"v{}.{}.{}",
			ash::vk::version_major(self.0),
			ash::vk::version_minor(self.0),
			ash::vk::version_patch(self.0)
		)
	}
}
impl From<u32> for VkVersion {
	fn from(v: u32) -> Self {
		VkVersion(v)
	}
}

/// Formats `[u8; 16]` as canonical `xxxxxxxx-xxxx-Mxxx-Nxxx-xxxxxxxxxxxx`.
pub fn format_uuid(uuid: [u8; 16]) -> impl Debug + Display {
	struct Inner {
		uuid: [u8; 16]
	}
	impl Debug for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result {
			write!(
				f,
				"{:0>2x}{:0>2x}{:0>2x}{:0>2x}-{:0>2x}{:0>2x}-{:0>2x}{:0>2x}-{:0>2x}{:0>2x}-{:0>2x}{:0>2x}{:0>2x}{:0>2x}{:0>2x}{:0>2x}",
				self.uuid[0], self.uuid[1], self.uuid[2], self.uuid[3],
				self.uuid[4], self.uuid[5],
				self.uuid[6], self.uuid[7],
				self.uuid[8], self.uuid[9],
				self.uuid[10], self.uuid[11], self.uuid[12], self.uuid[13], self.uuid[14], self.uuid[15]
			)
		}
	}
	impl Display for Inner {
		fn fmt(&self, f: &mut Formatter) -> Result {
			write!(
				f,
				"{:x}{:x}{:x}{:x}-{:x}{:x}-{:x}{:X}-{:x}{:x}-{:x}{:x}{:x}{:x}{:x}{:x}",
				self.uuid[0],
				self.uuid[1],
				self.uuid[2],
				self.uuid[3],
				self.uuid[4],
				self.uuid[5],
				self.uuid[6],
				self.uuid[7],
				self.uuid[8],
				self.uuid[9],
				self.uuid[10],
				self.uuid[11],
				self.uuid[12],
				self.uuid[13],
				self.uuid[14],
				self.uuid[15]
			)
		}
	}

	Inner { uuid }
}
