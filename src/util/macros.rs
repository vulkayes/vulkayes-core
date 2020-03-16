/// Generates a private enum and a public newtype struct that only has that enum as a value.
/// Also generated constructors on the struct that match the enum variants.
///
/// This is useful for making certain enum variants "unsafe" by only allowing their construction using
/// an unsafe function. Variants also may be private.
///
/// Structure enum variants are not supported.
#[macro_export]
macro_rules! unsafe_enum_variants {
	(
		$(#[$attribute: meta])*
		enum $inner_name: ident {
			$(
				$(#[$variant_attribute: meta])*
				$({$safety: tt})? $v: vis $variant: ident $( ($( $variant_data: ident ),+) )?
			),+
		} as pub $name: ident
	) => {
		$(#[$attribute])*
		enum $inner_name {
			$(
				$variant $( ($( $variant_data ),+) )?
			),+
		}
		$(#[$attribute])*
		pub struct $name($inner_name);
		impl $name {
			$(
				$(#[$variant_attribute])*
				#[allow(non_snake_case)]
				$v const $($safety)? fn $variant($( $( $variant_data: $variant_data ),+ )?) -> Self {
					$name(
						$inner_name::$variant $( ($( $variant_data ),+) )?
					)
				}
			)*
		}
	}
}

/// Generates a public enum that derives `thiserror::Error` with `VkResult` variants and their `From` impls.
#[macro_export]
macro_rules! vk_result_error {
	(
		$( #[$attribute: meta] )*
		pub enum $name: ident {
			vk {
				$(
					$( #[$variant_attribute: meta] )*
					$vk_error: ident
				),+
			}
			$( $other: tt )*
		}
	) => {
		#[allow(unused_imports)]
		use thiserror::*;

		$( #[$attribute] )*
		#[derive(Error)]
		pub enum $name {
			$(
				$( #[$variant_attribute] )*
				#[error("{}", ash::vk::Result::$vk_error)]
				#[allow(non_camel_case_types)]
				$vk_error,
			)+

			$( $other )*
		}
		impl From<ash::vk::Result> for $name {
			fn from(err: ash::vk::Result) -> Self {
				match err {
					$(
						ash::vk::Result::$vk_error => $name::$vk_error,
					)+
					_ => unreachable!("Cannot create {} from {}", stringify!($name), err) // TODO: Use unreachable unchecked in Release?
				}
			}
		}
		impl std::convert::TryInto<ash::vk::Result> for $name {
			type Error = $name;
			fn try_into(self) -> Result<ash::vk::Result, Self::Error> {
				#[allow(unreachable_patterns)]
				match self {
					$(
						$name::$vk_error => Ok(ash::vk::Result::$vk_error),
					)+
					_ => Err(self)
				}
			}
		}
	}
}

/// Implements `Deref`, `PartialEq`, `Eq` and `Hash` for a type based on its `Deref` implementation.
///
/// Since not all types deref directly into a handle, it is possible to provide a code fragment to get handle from deref target:
/// ```
/// impl_cmmon_handle_traits! {
/// 	impl [A: Debug] Deref, PartialEq, Eq, Hash for MyType [A] {
/// 		type Target = DerefTarget { field_on_self } // Derefs to `DerefTarget` by invoking `&self.field_on_self`
///
/// 		to_handle { .handle() } // Gets a handle from `DerefTarget` by invoking `self.field_on_self.handle()`
/// 	}
/// }
/// ```
///
/// this expands to
///
/// ```
/// impl<A: Debug> Deref for MyType<A> {
/// 	type Target = DerefTarget;
///
/// 	fn deref(&self) -> &Self::Target {
/// 		&self.field_on_self
/// 	}
/// }
/// impl<A: Debug> PartialEq for MyType<A> {
/// 	fn eq(&self, other: &Self) -> bool {
/// 		self.field_on_self.handle() == other.field_on_self.handle()
/// 	}
/// }
/// impl<A: Debug> Eq for MyType<A> {}
/// impl<A: Debug> Hash for MyType<A> {
/// 	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
/// 		self.field_on_self.handle().hash(state)
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! impl_common_handle_traits {
	(
		impl $([$($impl_gen: tt)*])? Deref, PartialEq, Eq, Hash for $tp: ty $([$($ty_gen: tt)*])? {
			type Target = $target: ty { $($target_code: tt)+ }

			$(
				to_handle { $($to_handle_code: tt)+ }
			)?
		}
	) => {
		impl $(<$($impl_gen)*>)? Deref for $tp $(<$($ty_gen)*>)? {
			type Target = $target;

			fn deref(&self) -> &Self::Target {
				&self.$($target_code)+
			}
		}
		impl $(<$($impl_gen)*>)? PartialEq for $tp $(<$($ty_gen)*>)? {
			fn eq(&self, other: &Self) -> bool {
				self.$($target_code)+ $( $($to_handle_code)+ )? == other.$($target_code)+ $( $($to_handle_code)+ )?
			}
		}
		impl $(<$($impl_gen)*>)? Eq for $tp $(<$($ty_gen)*>)? {}
		impl $(<$($impl_gen)*>)? std::hash::Hash for $tp $(<$($ty_gen)*>)? {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				self.$($target_code)+ $( $($to_handle_code)+ )? .hash(state)
			}
		}
	}
}

/// Creates a `repr(C)` struct and a companion offsets struct which represents byte offsets of the fields.
///
/// ```
/// offsetable_struct! {
/// 	#[derive(Debug)]
/// 	pub struct Name {
/// 		pub a: f32,
/// 		pub b: [f32; 4],
/// 		c: u8
/// 	} repr(C) as NameOffsets
/// }
/// ```
///
/// expands to
///
/// ```
/// #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
/// pub struct NameOffsets {
/// 	pub a: usize,
/// 	pub b: usize,
/// 	c: usize
/// }
///
/// #[derive(Debug)]
/// #[repr(C)]
/// pub struct Name {
/// 	pub a: f32,
/// 	pub b: [f32; 4],
/// 	c: u8
/// }
/// impl Name {
/// 	#[allow(unused_variables)]
/// 	pub const fn offsets() -> NameOffsets {
/// 		let current_offset: usize = 0;
///
/// 		let a = {
/// 			let x_minus_one = current_offset.wrapping_sub(1);
/// 			let alignment = std::mem::align_of::<f32>();
///
/// 			x_minus_one.wrapping_add(alignment).wrapping_sub(x_minus_one % alignment)
/// 		};
/// 		let current_offset = a + std::mem::size_of::<f32>();
///
/// 		let b = {
/// 			let x_minus_one = current_offset.wrapping_sub(1);
/// 			let alignment = std::mem::align_of::<[f32; 4]>();
///
/// 			x_minus_one.wrapping_add(alignment).wrapping_sub(x_minus_one % alignment)
/// 		};
/// 		let current_offset = b + std::mem::size_of::<[f32; 4]>();
///
/// 		let c = {
/// 			let x_minus_one = current_offset.wrapping_sub(1);
/// 			let alignment = std::mem::align_of::<u8>();
///
/// 			x_minus_one.wrapping_add(alignment).wrapping_sub(x_minus_one % alignment)
/// 		};
/// 		let current_offset = c + std::mem::size_of::<u8>();
///
/// 		NameOffsets {
/// 			a,
/// 			b,
/// 			c
/// 		}
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! offsetable_struct {
	(
		$( #[$attribute: meta] )*
		$struct_vis: vis struct $name: ident {
			$(
				$field_vis: vis $field: ident: $ftype: ty
			),*
		} repr(C) as $offsets_name: ident
	) => {
		#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
		$struct_vis struct $offsets_name {
			$(
				$field_vis $field: usize
			),*
		}

		$( #[$attribute] )*
		#[repr(C)]
		$struct_vis struct $name {
			$(
				$field_vis $field: $ftype
			),*
		}
		impl $name {
			/// Returns a struct describing offsets of each field from the start of the struct.
			///
			/// This is mainly useful for things like vertex data
			#[allow(unused_variables)]
			pub const fn offsets() -> $offsets_name {
				let current_offset: usize = 0;

				$(
					// ```
					// f_d(x) =
					//     0, if x mod d = 0
					//     d - x mod d, otherwise
					// ```
					// simplifies to `x - 1 + d - (x - 1) mod d`
					// assuming `d = 2^N`, can also be written in code like: `(x - 1 + d) & !(d - 1)`
					//
					// Similar code to `std::alloc::Layout::padding_needed_for`
					let $field = {
						let x_minus_one = current_offset.wrapping_sub(1);
						let alignment = std::mem::align_of::<$ftype>();

						x_minus_one.wrapping_add(alignment).wrapping_sub(x_minus_one % alignment)
					};
					// let $field = {
					// 	let alignment_minus_one = std::mem::align_of::<$ftype>().wrapping_sub(1);
					//
					// 	current_offset.wrapping_add(alignment_minus_one) & !alignment_minus_one
					// };
					let current_offset = current_offset + std::mem::size_of::<$ftype>();
				)*

				$offsets_name {
					$(
						$field
					),*
				}
			}
		}
	}
}

/// Creates two fixed-size arrays. The first one holds locks and the second one holds deref of those locks.
///
/// Usage:
/// ```
/// lock_and_deref_closure!(
/// 	let foo[2]{.lock().unwrap()} => |foo_locks, foo_derefs|
/// 	let bar[0]{.lock().unwrap()} => |bar_locks: [LockGuard<Bar>; 0], bar_derefs|
/// 	{
/// 		println!("{:?} {:?}", foo_derefs, bar_derefs);
/// 	}
/// )
/// ```
/// expands to
/// ```
/// {
/// 	let (foo_locks, foo_derefs) = {
/// 		let locks = [foo[0].lock().unwrap(), foo[1].lock().unwrap()];
/// 		let derefs = [*locks[0], *locks[1]];
///
/// 		(locks, derefs)
/// 	};
/// 	let (bar_locks, bar_derefs) = {
/// 		let locks = [];
/// 		let derefs = [];
///
/// 		(locks, derefs)
/// 	};
///
/// 	let closure = |foo_locks: [_; 2], foo_derefs: [_; 2], bar_locks: [_; 0], bar_derefs: [_; 0]| { println!("{:?} {:?}", foo_derefs, bar_derefs); };
/// 	closure(foo_locks, foo_derefs, bar_locks, bar_derefs)
/// }
/// ```
///
/// This macro uses a `proc-macro-hack` version of the `seq-macro` crate to generate the array indices.
#[macro_export]
macro_rules! lock_and_deref_closure {
	(
		$(
			let $ex: ident[$count: literal] {$($lock_code: tt)+} => |$locks: ident $(: $l_type: ty)?, $derefs: ident|
		)+
		{ $($closure_body: tt)* }
	) => {
		{
			$(
				let ($locks, $derefs) = $crate::seq_macro::seq_expr!(
					N in 0 .. $count {
						{
							let locks $(: $l_type)? = [ #( $ex[N] $($lock_code)+, )* ];
							let derefs = [ #( *locks[N], )* ];

							(locks, derefs)
						}
					}
				);
			)+

			let closure = |$( $locks: [_; $count], $derefs: [_; $count] ),+| { $($closure_body)* };
			closure($( $locks, $derefs ),+)
		}
	}
}
