//! Macros galore!

/// Generates a private enum and a public newtype struct that only has that enum as a value.
/// Also generated constructors on the struct that match the enum variants.
///
/// This is useful for making certain enum variants "unsafe" by only allowing their construction using
/// an unsafe function. Variants may also be private.
///
/// Since a common usecase across this crate for this macro are typesafe parameter combinations, there
/// is also a version with `Into` implementation.
///
/// Tuple enum variants are not supported.
///
/// Usage:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// unsafe_enum_variants! {
/// 	#[derive(Debug)]
/// 	enum UnsafeEnumInner ['a] {
/// 		/// Private
/// 		Foo => { &0u32 },
/// 		/// Public
/// 		pub Bar => { &1u32 },
/// 		/// Unsafe and generic
/// 		{unsafe} pub Qux { num: &'a u32 } => { num }
/// 	} as pub UnsafeEnum ['a] impl Into<&'a u32>
/// }
/// ```
///
/// expands to:
/// ```
/// #[derive(Debug)]
/// enum UnsafeEnumInner<'a> {
/// 	Foo,
/// 	Bar,
/// 	Qux {
/// 		num: &'a u32
/// 	}
/// }
/// #[derive(Debug)]
/// pub struct UnsafeEnum<'a>(UnsafeEnumInner<'a>);
/// impl<'a> UnsafeEnum<'a> {
/// 	#[doc = r###"Private"###]
/// 	#[allow(non_snake_case)]
/// 	const fn Foo() -> Self {
/// 		UnsafeEnum(UnsafeEnumInner::Foo)
/// 	}
/// 	#[doc = r###"Public"###]
/// 	#[allow(non_snake_case)]
/// 	pub const fn Bar() -> Self {
/// 		UnsafeEnum(UnsafeEnumInner::Bar)
/// 	}
/// 	#[doc = r###"Unsafe"###]
/// 	#[allow(non_snake_case)]
/// 	pub const unsafe fn Qux(num: &'a u32) -> Self {
/// 		UnsafeEnum(UnsafeEnumInner::Qux { num })
/// 	}
/// }
/// impl<'a> Into<&'a u32> for UnsafeEnum<'a> {
/// 	fn into(self) -> &'a u32 {
/// 		match self.0 {
/// 			UnsafeEnumInner::Foo => { &0u32 },
/// 			UnsafeEnumInner::Bar => { &1u32 },
/// 			UnsafeEnumInner::Qux { num } => { num }
/// 		}
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! unsafe_enum_variants {
	(
		$(#[$attribute: meta])*
		enum $inner_name: ident $([ $($generic_bounds: tt)+ ])? {
			$(
				$(#[$variant_attribute: meta])*
				$({$safety: tt})? $v: vis $variant: ident $({
					 $($variant_name: ident: $variant_type: ty),+ $(,)?
				})? => { $($into_code: tt)+ }
			),+ $(,)?
		} as pub $name: ident $([ $($generic_params: tt)+ ])? impl Into<$into_type: ty>
	) => {
		unsafe_enum_variants!(
			$(#[$attribute])*
			enum $inner_name $([ $($generic_bounds)+ ])? {
				$(
					$(#[$variant_attribute])*
					$({$safety})? $v $variant $({ $($variant_name: $variant_type),+ })?
				),+
			} as pub $name $([ $($generic_params)+ ])?
		);
		impl $(< $($generic_bounds)+ >)? Into<$into_type> for $name $(< $($generic_params)+ >)? {
			fn into(self) -> $into_type {
				#[allow(unused_doc_comments)]
				match self.0 {
					$(
						$(#[$variant_attribute])*
						$inner_name::$variant $({ $($variant_name),+ })? => { $($into_code)+ }
					),+
				}
			}
		}
	};

	(
		$(#[$attribute: meta])*
		enum $inner_name: ident $([ $($generic_bounds: tt)+ ])? {
			$(
				$(#[$variant_attribute: meta])*
				$({$safety: tt})? $v: vis $variant: ident $({
					 $($variant_name: ident: $variant_type: ty),+ $(,)?
				})?
			),+ $(,)?
		} as pub $name: ident $([ $($generic_params: tt)+ ])?
	) => {
		$(#[$attribute])*
		enum $inner_name $(< $($generic_bounds)+ >)? {
			$(
				$(#[$variant_attribute])*
				$variant $({
					 $($variant_name: $variant_type),+
				})?
			),+
		}
		$(#[$attribute])*
		pub struct $name $(< $($generic_bounds)+ >)? ($inner_name $(< $($generic_params)+ >)?);
		impl $(< $($generic_bounds)+ >)? $name $(< $($generic_params)+ >)? {
			$(
				$(#[$variant_attribute])*
				#[allow(non_snake_case)]
				$v const $($safety)? fn $variant($( $( $variant_name: $variant_type ),+ )?) -> Self {
					$name(
						#[allow(unused_doc_comments)]
						$(#[$variant_attribute])*
						$inner_name::$variant $({ $($variant_name),+ })?
					)
				}
			)*
		}
	};
}

/// Wraps an ash builder in a `#[repr(transparent)]` struct.
///
/// Usage:
/// ```
/// # use vulkayes_core::vk_builder_wrap;
/// # #[repr(transparent)]
/// # pub struct BuilderType<'a>(BuilderTargetType, std::marker::PhantomData<&'a ()>);
/// # impl<'a> std::ops::Deref for BuilderType<'a> { type Target = BuilderTargetType; fn deref(&self) -> &Self::Target { &self.0 } }
/// # #[derive(Debug)]
/// # pub struct BuilderTargetType(u32);
///
/// vk_builder_wrap! {
/// 	/// Doc comment
/// 	pub struct Foo ['a] {
/// 		// the `=> BuilderTargetType` part is optional and generates an additional Transparent unsafe impl
/// 		builder: BuilderType<'a> => BuilderTargetType
/// 	}
/// 	impl ['a] {
/// 		pub fn new(param: &'a u32) -> Self {
/// 			todo!()
/// 		}
/// 	}
/// }
/// ```
///
/// expands to:
/// ```
/// # #[repr(transparent)]
/// # pub struct BuilderType<'a>(BuilderTargetType, std::marker::PhantomData<&'a ()>);
/// # impl<'a> std::ops::Deref for BuilderType<'a> { type Target = BuilderTargetType; fn deref(&self) -> &Self::Target { &self.0 } }
/// # #[derive(Debug)]
/// # pub struct BuilderTargetType(u32);
///
/// #[doc = r###"Doc comment"###]
/// #[repr(transparent)]
/// pub struct Foo<'a> {
/// 	builder: BuilderType<'a>
/// }
/// impl<'a> Foo<'a> {
/// 	pub const unsafe fn from_raw(
/// 		builder: BuilderType<'a>
/// 	) -> Self {
/// 		Foo {
/// 			builder
/// 		}
/// 	}
///
/// 	pub fn new(param: &'a u32) -> Self { todo!() }
/// }
/// impl<'a> std::ops::Deref for Foo<'a> {
/// 	type Target = BuilderType<'a>;
///
/// 	fn deref(&self) -> &Self::Target {
/// 		&self.builder
/// 	}
/// }
/// impl<'a> std::ops::DerefMut for Foo<'a> {
/// 	fn deref_mut(&mut self) -> &mut Self::Target {
/// 		&mut self.builder
/// 	}
/// }
/// unsafe impl<'a> vulkayes_core::util::transparent::Transparent for Foo<'a> {
/// 	type Target = BuilderType<'a>;
/// }
/// // This is optional
/// unsafe impl<'a> vulkayes_core::util::transparent::Transparent for BuilderType<'a> {
/// 	type Target = BuilderTargetType
/// 	;
/// }
/// impl<'a> std::fmt::Debug for Foo<'a> {
/// 	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
/// 		f.debug_struct(stringify!( Foo ))
/// 			.field("self.builder.deref()", std::ops::Deref::deref(&self.builder))
/// 			.finish()
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! vk_builder_wrap {
	(
		$(#[$attribute: meta])*
		pub struct $name: ident $([ $($generic_bounds: tt)+ ])? {
			builder: $target: ty $(=> $vk_target: ty)?
		}
		impl $([ $($generic_params: tt)+ ])? {
			$(
				$impl_code: tt
			)+
		}
	) => {
		$(#[$attribute])*
		#[repr(transparent)]
		pub struct $name $(< $($generic_bounds)+ >)? {
			builder: $target
		}
		impl $(< $($generic_bounds)+ >)? $name $(< $($generic_params)+ >)? {
			pub const unsafe fn from_raw(
				builder: $target
			) -> Self {
				$name {
					builder
				}
			}

			$(
				$impl_code
			)+
		}
		impl $(< $($generic_bounds)+ >)? std::ops::Deref for $name $(< $($generic_params)+ >)? {
			type Target = $target;

			fn deref(&self) -> &Self::Target {
				&self.builder
			}
		}
		impl $(< $($generic_bounds)+ >)? std::ops::DerefMut for $name $(< $($generic_params)+ >)? {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.builder
			}
		}
		unsafe impl $(< $($generic_bounds)+ >)? $crate::util::transparent::Transparent for $name $(< $($generic_params)+ >)? {
			type Target = $target;
		}
		$(
			unsafe impl<'a> $crate::util::transparent::Transparent for $target {
				type Target = $vk_target;
			}
		)?
		impl $(< $($generic_bounds)+ >)? std::fmt::Debug for $name $(< $($generic_params)+ >)? {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.debug_struct(stringify!($name))
					.field("self.builder.deref()", std::ops::Deref::deref(&self.builder))
				.finish()
			}
		}
	}
}

/// Generates a public enum that derives `thiserror::Error` with `VkResult` variants and their `From` impls.
///
/// Usage:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// trait Trait: std::error::Error + 'static {}
///
/// vk_result_error! {
/// 	#[derive(Debug)]
/// 	pub enum ImageError [A] where [A: Trait] {
/// 		vk {
/// 			ERROR_OUT_OF_HOST_MEMORY,
/// 			ERROR_OUT_OF_DEVICE_MEMORY
/// 		}
///
/// 		#[error("Description")]
/// 		Other(#[from] A)
/// 	}
/// }
/// ```
///
/// expands to:
/// ```
/// # trait Trait: std::error::Error + 'static {}
/// // ...
///
/// #[allow(unused_imports)]
/// use thiserror::*;
///
/// #[derive(Debug)]
/// #[derive(Error)]
/// pub enum ImageError<A: Trait> {
/// 	#[error("{}", ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY)]
/// 	#[allow(non_camel_case_types)]
/// 	ERROR_OUT_OF_HOST_MEMORY,
/// 	#[error("{}", ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY)]
/// 	#[allow(non_camel_case_types)]
/// 	ERROR_OUT_OF_DEVICE_MEMORY,
///
/// 	#[error("Description")]
/// 	Other(#[from] A)
/// }
/// impl<A: Trait> From<ash::vk::Result> for ImageError<A> {
/// 	fn from(err: ash::vk::Result) -> Self {
/// 		match err {
/// 			ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => ImageError::ERROR_OUT_OF_HOST_MEMORY,
/// 			ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => ImageError::ERROR_OUT_OF_DEVICE_MEMORY,
/// 			_ => unreachable!("Cannot create {} from {}", stringify!(ImageError), err)
/// 		}
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! vk_result_error {
	(
		$( #[$attribute: meta] )*
		pub enum $name: ident $([ $($generic_params: tt)+ ] where [ $($generic_bounds: tt)+ ])? {
			vk {
				$(
					$( #[$variant_attribute: meta] )*
					$vk_error: ident
				),+ $(,)?
			}
			$( $other: tt )*
		}
	) => {
		#[allow(unused_imports)]
		use thiserror::*;

		$( #[$attribute] )*
		#[derive(Error)]
		pub enum $name $(< $($generic_bounds)+ >)? {
			$(
				$( #[$variant_attribute] )*
				#[error("{}", ash::vk::Result::$vk_error)]
				#[allow(non_camel_case_types)]
				$vk_error,
			)+

			$( $other )*
		}
		impl $(< $($generic_bounds)+ >)? From<ash::vk::Result> for $name $(< $($generic_params)+ >)?  {
			fn from(err: ash::vk::Result) -> Self {
				match err {
					$(
						ash::vk::Result::$vk_error => $name::$vk_error,
					)+
					_ => unreachable!("Cannot create {} from {}", stringify!($name), err)
				}
			}
		}
	}
}

/// Implements `Borrow`, `Deref`, `PartialEq`, `Eq`, `Hash`, `PartialOrd` and `Ord` for a type based on its `Borrow` implementation.
///
/// This macro is closely tied to the `HasHandle` and `HasSynchronizedHandle` traits.
///
/// There are three variants of this macro:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// # use std::fmt::Debug;
/// # use vulkayes_core::prelude::Vutex;
/// # use vulkayes_core::ash::vk;
/// #
/// # struct Foo;
/// # impl Foo {
/// # 	fn handle(&self) -> vk::Image {
/// # 		unimplemented!()
/// # 	}
/// # }
/// # type Target = Foo;
///
/// struct MyType<A> {
/// 	field_on_self: Target,
/// 	other_field: A
/// }
///
/// // Base variant
/// // Deref is optional. If it is not desired, the `<Target>` token is appended to `Borrow` instead.
/// impl_common_handle_traits! {
/// 	impl [A: Debug] Deref<Target>, Borrow, Eq, Hash, Ord for MyType<A> {
/// 		target = { field_on_self } // Borrows and Derefs to `Target` by invoking `&self.field_on_self`
///
/// 		to_handle { .handle() } // Gets a handle from `Target` by invoking `self.field_on_self.handle()`
/// 	}
/// }
///
/// // HasHandle variant
/// // struct MyType<A> {
/// // 	field_on_self: vk::Image,
/// // 	other_field: A
/// // }
/// // impl_common_handle_traits! {
/// // 	impl [A: Debug] HasHandle<Target>, Deref, Borrow, Eq, Hash, Ord for MyType<A> {
/// // 		target = { field_on_self }
/// // 	}
/// // }
///
/// // HasSynchronizedHandle variant
/// // struct MyType<A> {
/// // 	field_on_self: Vutex<vk::Image>,
/// // 	other_field: A
/// // }
/// // impl_common_handle_traits! {
/// // 	impl [A: Debug] HasSynchronizedHandle<Target>, Deref, Borrow, Eq, Hash, Ord for MyType<A> {
/// // 		target = { field_on_self }
/// // 	}
/// // }
/// ```
///
/// expands to:
/// ```
/// # use std::fmt::Debug;
/// # use vulkayes_core::prelude::Vutex;
/// # use vulkayes_core::ash::vk;
/// #
/// # struct Foo;
/// # impl Foo {
/// # 	fn handle(&self) -> vk::Image {
/// # 		unimplemented!()
/// # 	}
/// # }
/// # type Target = Foo;
/// #
/// # struct MyType<A> {
/// # 	field_on_self: Target,
/// # 	other_field: A
/// # }
/// // ...
///
/// // Base variant
/// // Deref is optional
/// impl<A: Debug> std::ops::Deref for MyType<A> {
/// 	type Target = Target;
///
/// 	fn deref(&self) -> &Self::Target {
/// 		&self.field_on_self
/// 	}
/// }
/// impl<A: Debug> std::borrow::Borrow<Target> for MyType<A> {
/// 	fn borrow(&self) -> &Target {
/// 		&self.field_on_self
/// 	}
/// }
///
/// impl<A: Debug> PartialEq for MyType<A> {
/// 	fn eq(&self, other: &Self) -> bool {
/// 		self.field_on_self.handle() == other.field_on_self.handle()
/// 	}
/// }
/// impl<A: Debug> Eq for MyType<A> {}
/// impl<A: Debug> std::hash::Hash for MyType<A> {
/// 	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
/// 		self.field_on_self.handle().hash(state)
/// 	}
/// }
///
/// impl<A: Debug> std::cmp::PartialOrd for MyType<A> {
/// 	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
/// 		self.field_on_self.handle().partial_cmp(&other.field_on_self.handle())
/// 	}
/// }
/// impl<A: Debug> std::cmp::Ord for MyType<A> {
/// 	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
/// 		self.field_on_self.handle().cmp(&other.field_on_self.handle())
/// 	}
/// }
///
/// // HasHandle variant adds to the previous implementations also:
/// // impl<A: Debug> vulkayes_core::util::handle::HasHandle<vk::Image> for MyType<A> {}
///
/// // While HasSynchronizedHandle adds:
/// // impl<A: Debug> vulkayes_core::util::handle::HasSynchronizedHandle<vk::Image> for MyType<A> {}
/// ```
#[macro_export]
macro_rules! impl_common_handle_traits {
	(
		impl $([ $($impl_gen: tt)+ ])? HasHandle<$target: ty>, Deref, Borrow, Eq, Hash, Ord for $tp: ty {
			target = { $($target_code: tt)+ }
		} $(+ $deref: ident)?
	) => {
		impl_common_handle_traits!(
			impl $([ $($impl_gen)+ ])? Deref<$target>, Borrow, Eq, Hash, Ord for $tp {
				target = { $($target_code)+ }
			}
		);
		impl $crate::util::handle::HasHandle<$target> for $tp {}
	};
	(
		impl $([ $($impl_gen: tt)+ ])? HasHandle<$target: ty>, Borrow, Eq, Hash, Ord for $tp: ty {
			target = { $($target_code: tt)+ }
		}
	) => {
		impl_common_handle_traits!(
			impl $([ $($impl_gen)+ ])? Borrow<$target>, Eq, Hash, Ord for $tp {
				target = { $($target_code)+ }
			}
		);
		impl $crate::util::handle::HasHandle<$target> for $tp {}
	};

	(
		impl $([ $($impl_gen: tt)+ ])? HasSynchronizedHandle<$target: ty>, Deref, Borrow, Eq, Hash, Ord for $tp: ty {
			target = { $($target_code: tt)+ }
		}
	) => {
		impl_common_handle_traits!(
			impl $([ $($impl_gen)+ ])? Deref<$crate::util::sync::Vutex<$target>>, Borrow, Eq, Hash, Ord for $tp {
				target = { $($target_code)+ }

				to_handle { .lock().expect("vutex poisoned").deref() }
			}
		);
		impl $crate::util::handle::HasSynchronizedHandle<$target> for $tp {}
	};
	(
		impl $([ $($impl_gen: tt)+ ])? HasSynchronizedHandle<$target: ty>, Borrow, Eq, Hash, Ord for $tp: ty {
			target = { $($target_code: tt)+ }
		}
	) => {
		impl_common_handle_traits!(
			impl $([ $($impl_gen)+ ])? Borrow<$crate::util::sync::Vutex<$target>>, Eq, Hash, Ord for $tp {
				target = { $($target_code)+ }

				to_handle { .lock().expect("vutex poisoned").deref() }
			}
		);
		impl $crate::util::handle::HasSynchronizedHandle<$target> for $tp {}
	};

	(
		impl $([ $($impl_gen: tt)+ ])? Deref<$target: ty>, Borrow, Eq, Hash, Ord for $tp: ty {
			target = { $($target_code: tt)+ }

			$(
				to_handle { $($to_handle_code: tt)+ }
			)?
		}
	) => {
		impl $(< $($impl_gen)+ >)? std::ops::Deref for $tp {
			type Target = $target;

			fn deref(&self) -> &Self::Target {
				&self.$($target_code)+
			}
		}
		impl_common_handle_traits!(
			impl $([ $($impl_gen)+ ])? Borrow<$target>, Eq, Hash, Ord for $tp {
				target = { $($target_code)+ }

				$(
					to_handle { $($to_handle_code)+ }
				)?
			}
		);
	};

	(
		impl $([ $($impl_gen: tt)+ ])? Borrow<$target: ty>, Eq, Hash, Ord for $tp: ty {
			target = { $($target_code: tt)+ }

			$(
				to_handle { $($to_handle_code: tt)+ }
			)?
		}
	) => {
		impl $(< $($impl_gen)+ >)? std::borrow::Borrow<$target> for $tp {
			fn borrow(&self) -> &$target {
				&self.$($target_code)+
			}
		}

		impl $(< $($impl_gen)+ >)? PartialEq for $tp {
			fn eq(&self, other: &Self) -> bool {
				self.$($target_code)+$( $($to_handle_code)+ )? == other.$($target_code)+$( $($to_handle_code)+ )?
			}
		}
		impl $(< $($impl_gen)+ >)? Eq for $tp {}
		impl $(< $($impl_gen)+ >)? std::hash::Hash for $tp {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				self.$($target_code)+$( $($to_handle_code)+ )?.hash(state)
			}
		}

		impl $(< $($impl_gen)+ >)? std::cmp::PartialOrd for $tp {
			fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
				self.$($target_code)+$( $($to_handle_code)+ )?.partial_cmp(&other.$($target_code)+$( $($to_handle_code)+ )?)
			}
		}
		impl $(< $($impl_gen)+ >)? std::cmp::Ord for $tp {
			fn cmp(&self, other: &Self) -> std::cmp::Ordering {
				self.$($target_code)+$( $($to_handle_code)+ )?.cmp(&other.$($target_code)+$( $($to_handle_code)+ )?)
			}
		}
	}
}

/// Creates a `repr(C)` struct and a companion offsets struct which represents byte offsets of the fields.
///
/// ```
/// # #[macro_use] extern crate vulkayes_core;
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
/// expands to:
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
/// 		let a = vulkayes_core::util::align_up(current_offset, std::mem::align_of::<f32>());
/// 		let current_offset = a + std::mem::size_of::<f32>();
///
/// 		let b = vulkayes_core::util::align_up(current_offset, std::mem::align_of::<[f32; 4]>());
/// 		let current_offset = b + std::mem::size_of::<[f32; 4]>();
///
/// 		let c = vulkayes_core::util::align_up(current_offset, std::mem::align_of::<u8>());
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
					let $field = $crate::util::align_up(current_offset, std::mem::align_of::<$ftype>());
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
/// This macro uses a `proc-macro-hack` version of the `seq-macro` crate to generate the array indices.
///
/// Usage:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// # use vulkayes_core::util::sync::{Vutex, VutexGuard};
/// # #[derive(Debug)]
/// # struct Bar;
/// let foo = [Vutex::new(0), Vutex::new(1)];
/// let bar: [Vutex<Bar>; 0] = [];
/// lock_and_deref_closure!(
/// 	let [foo; 2]{.lock().unwrap()} => |foo_locks, foo_derefs|
/// 	let [bar; 0]{.lock().unwrap()} => |bar_locks: [VutexGuard<Bar>; 0], bar_derefs|
/// 	{
/// # 		let bar_derefs: [&Bar; 0] = bar_derefs;
/// 		println!("{:?} {:?}", foo_derefs, bar_derefs);
/// 	}
/// )
/// ```
///
/// expands to:
/// ```
/// # use vulkayes_core::util::sync::{Vutex, VutexGuard};
/// # #[derive(Debug)]
/// # struct Bar;
/// # let foo = [Vutex::new(0), Vutex::new(1)];
/// # let bar: [Vutex<Bar>; 0] = [];
/// {
/// 	let (foo_locks, foo_derefs) = {
/// 		let locks = [foo[0].lock().unwrap(), foo[1].lock().unwrap()];
/// 		let derefs = [*locks[0], *locks[1]];
///
/// 		(locks, derefs)
/// 	};
/// 	let (bar_locks, bar_derefs) = {
/// 		let locks: [VutexGuard<Bar>; 0] = [];
/// 		let derefs = [];
///
/// 		(locks, derefs)
/// 	};
///
/// 	let closure = |foo_locks: [_; 2], foo_derefs: [_; 2], bar_locks: [_; 0], bar_derefs: [_; 0]| {
/// # 		let bar_derefs: [&Bar; 0] = bar_derefs;
/// 		println!("{:?} {:?}", foo_derefs, bar_derefs);
/// 	};
/// 	closure(foo_locks, foo_derefs, bar_locks, bar_derefs)
/// }
/// ```
#[macro_export]
macro_rules! lock_and_deref_closure {
	(
		$(
			let [$ex: ident; $count: literal] {$($lock_code: tt)+} => |$locks: ident $(: $l_type: ty)?, $derefs: ident|
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

/// Simple enum dispatch using `Deref`. Suitable for mixed dispatch enums.
///
/// Usage:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// use std::{ops::Deref, rc::Rc};
/// pub struct Baz;
///
/// #[derive(Debug, Clone)]
/// pub struct Foo;
/// impl Deref for Foo {
/// 	type Target = Baz;
/// #
/// # 	fn deref(&self) -> &Self::Target {
/// # 		&Baz
/// # 	}
/// }
/// #[derive(Debug, Clone)]
/// pub struct Bar;
/// impl Deref for Bar {
/// 	type Target = Baz;
/// #
/// # 	fn deref(&self) -> &Self::Target {
/// # 		&Baz
/// # 	}
/// }
/// trait Trait: Deref<Target = Baz> + std::fmt::Debug {}
///
/// deref_enum_dispatch! {
/// 	/// Mixed-dispatch image enum.
/// 	#[derive(Debug, Clone)]
/// 	pub enum MixedDynTrait {
/// 		Foo(Foo),
/// 		Bar(Bar),
/// 		Dyn(Rc<dyn Trait>)
/// 	}: Deref<Target = Baz>
/// }
/// ```
///
/// expands to:
/// ```
/// # use std::{ops::Deref, rc::Rc};
/// # pub struct Baz;
/// # #[derive(Debug, Clone)]
/// # pub struct Foo;
/// # impl Deref for Foo {
/// # 	type Target = Baz;
/// #
/// # 	fn deref(&self) -> &Self::Target {
/// # 		&Baz
/// # 	}
/// # }
/// # #[derive(Debug, Clone)]
/// # pub struct Bar;
/// # impl Deref for Bar {
/// # 	type Target = Baz;
/// #
/// # 	fn deref(&self) -> &Self::Target {
/// # 		&Baz
/// # 	}
/// # }
/// # trait Trait: Deref<Target = Baz> + std::fmt::Debug {}
/// // ...
///
/// /// Mixed-dispatch image enum.
/// #[derive(Debug, Clone)]
/// pub enum MixedDynTrait {
/// 	Foo(Foo),
/// 	Bar(Bar),
/// 	Dyn(Rc<dyn Trait>)
/// }
/// impl Deref for MixedDynTrait {
/// 	type Target = Baz;
///
/// 	fn deref(&self) -> &Self::Target {
/// 		match self {
/// 			MixedDynTrait::Foo(value) => value.deref(),
/// 			MixedDynTrait::Bar(value) => value.deref(),
/// 			MixedDynTrait::Dyn(value) => value.deref()
/// 		}
/// 	}
/// }
/// impl From<Foo> for MixedDynTrait {
/// 	fn from(value: Foo) -> Self {
/// 		MixedDynTrait::Foo(value)
/// 	}
/// }
/// impl From<Bar> for MixedDynTrait {
/// 	fn from(value: Bar) -> Self {
/// 		MixedDynTrait::Bar(value)
/// 	}
/// }
/// impl From<Rc<dyn Trait>> for MixedDynTrait {
/// 	fn from(value: Rc<dyn Trait>) -> Self {
/// 		MixedDynTrait::Dyn(value)
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! deref_enum_dispatch {
	(
		$( #[$attribute: meta] )*
		$visibility: vis enum $name: ident {
			$(
				$variant: ident ($variant_payload: ty)
			),+
		}: Deref<Target = $target: ty>
	) => {
		$( #[$attribute] )*
		$visibility enum $name {
			$(
				$variant ($variant_payload)
			),+
		}
		impl Deref for $name {
			type Target = $target;

			fn deref(&self) -> &Self::Target {
				match self {
					$(
						$name::$variant(value) => value.deref()
					),+
				}
			}
		}
		$(
			impl From<$variant_payload> for $name {
				fn from(value: $variant_payload) -> Self {
					$name::$variant(value)
				}
			}
		)+
	}
}

/// Creates a subset of a vk enum, which is defined as an i32 struct with associated constants.
///
/// Usage:
/// ```
/// # use vulkayes_core::vk_enum_subset;
/// # mod vk {
/// # 	#[derive(Debug, Eq, PartialEq)]
/// # 	pub struct MainEnum(i32);
/// # 	impl MainEnum {
/// # 		pub const FOO: Self = MainEnum(0);
/// # 		pub const BAR: Self = MainEnum(1);
/// # 		pub const BAZ: Self = MainEnum(2);
/// # 		pub const QUZ: Self = MainEnum(3);
/// #
/// # 		pub const fn as_raw(self) -> i32 { self.0 }
/// # 		pub const fn from_raw(v: i32) -> Self { MainEnum(v) }
/// # 	}
/// # }
///
/// vk_enum_subset! {
/// 	/// Doc
/// 	pub enum SubsetEnum {
/// 		FOO,
/// 		BAR,
/// 		BAZ
/// 	} impl Into<vk::MainEnum>
/// }
/// ```
///
/// expands to:
/// ```
/// # mod vk {
/// # 	#[derive(Debug, Eq, PartialEq)]
/// # 	pub struct MainEnum(i32);
/// # 	impl MainEnum {
/// # 		pub const FOO: Self = MainEnum(0);
/// # 		pub const BAR: Self = MainEnum(1);
/// # 		pub const BAZ: Self = MainEnum(2);
/// # 		pub const QUZ: Self = MainEnum(3);
/// #
/// # 		pub const fn as_raw(self) -> i32 { self.0 }
/// # 		pub const fn from_raw(v: i32) -> Self { MainEnum(v) }
/// # 	}
/// # }
///
/// #[allow(non_camel_case_types)]
/// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// #[repr(i32)]
/// /// Doc
/// pub enum SubsetEnum {
/// 	FOO = <vk::MainEnum>::FOO.as_raw(),
/// 	BAR = <vk::MainEnum>::BAR.as_raw(),
/// 	BAZ = <vk::MainEnum>::BAZ.as_raw()
/// }
/// impl Into<vk::MainEnum> for SubsetEnum {
/// 	fn into(self) -> vk::MainEnum {
/// 		<vk::MainEnum>::from_raw(self as i32)
/// 	}
/// }
/// impl std::convert::TryFrom<vk::MainEnum> for SubsetEnum {
/// 	type Error = String;
///
/// 	fn try_from(value: vk::MainEnum) -> Result<Self, Self::Error> {
/// 		match value {
/// 			vk::MainEnum::FOO => Ok(SubsetEnum::FOO),
/// 			vk::MainEnum::BAR => Ok(SubsetEnum::BAR),
/// 			vk::MainEnum::BAZ => Ok(SubsetEnum::BAZ),
/// 			_ => Err(
/// 				format!(
/// 					concat!("Cannot convert from ", stringify!(vk::MainEnum), "::{:?} to ", stringify!(SubsetEnum)),
/// 					value
/// 				)
/// 			)
/// 		}
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! vk_enum_subset {
	(
		$( #[$attribute: meta] )*
		pub enum $name: ident {
			$(
				$( #[$variant_attribute: meta] )*
				$variant: ident
			),+ $(,)?
		} impl Into<$vk_enum: ty>
	) => {
		#[allow(non_camel_case_types)]
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
		#[repr(i32)]
		$( #[$attribute] )*
		pub enum $name {
			$(
				$( #[$variant_attribute] )*
				$variant = <$vk_enum>::$variant.as_raw()
			),+
		}
		impl Into<$vk_enum> for $name {
			fn into(self) -> $vk_enum {
				<$vk_enum>::from_raw(self as i32)
			}
		}
		impl std::convert::TryFrom<$vk_enum> for $name {
			type Error = String;

			fn try_from(value: $vk_enum) -> Result<Self, Self::Error> {
				match value {
					$( <$vk_enum>::$variant => Ok($name::$variant), )+
					_ => Err(
						format!(
							concat!("Cannot convert from ", stringify!($vk_enum), "::{:?} to ", stringify!($name)),
							value
						)
					)
				}
			}
		}
	}
}

/// Collect an iterator into a chosen type.
///
/// This macro is mainly provided so that collect implementation used across the crate to lower memory allocation overhead
/// can be swapped and benchmarked easily. Implementations may come and go and this macro will likely be completely removed at some point.
///
/// `$static_size_hint` should be a const-evaluated expression hinting how much memory to preallocate:
/// * For `Vec` implementation, this is ignored.
/// * For `smallvec` implementation, this chooses the size of the `SmallVec` stack array.
/// * For `reusable-memory` implementation, this chooses the size of the borrow.
/// * For `bumpalo` implementation, this is ignored.
///
/// `global_state_access` should be an expression:
/// * For `Vec` implementation, this is ignored.
/// * For `smallvec` implemnetation, this is ignored.
/// * For `reusable-memory` implementation, this provides access to the `ReusableMemory` object to borrow from.
/// * For `bumpalo` implementation, this provides access to `Bump` object.
macro_rules! collect_iter_faster {
	(
		@vec
		$(
			$iter: expr, $static_size_hint: expr
		),+
	) => {
		(
			$(
				$iter.collect::<
					Vec<_>
				>()
			),+
		)
	};

	(
		@smallvec
		$(
			$iter: expr, $static_size_hint: expr
		),+
	) => {
		(
			$(
				$iter.collect::<
					$crate::smallvec::SmallVec<
						[_; $static_size_hint]
					>
				>()
			),+
		)
	};

	// TODO: These are not trivial to macroize
	// This is tricky. The `$global_state` needs to give us the correct function to call as well such as `borrow_mut_two_as`
	// (
	// 	@reusable_memory $global_state_access: expr;
	// 	$(
	// 		$iter: expr, $static_size_hint: expr
	// 	),+
	// ) => {
	// 	{
	// 		$global_state_access::<
	// 			$(
	// 				<$iter as Iterator>::Item
	// 			),+
	// 		>(
	// 			[
	// 				$(
	// 					std::num::NonZeroUsize::new($static_size_hint).unwrap()
	// 				),+
	// 			]
	// 		)
	// 	}
	// };
	//
	// (
	// 	@bumpalo $global_state_access: expr;
	// 	$(
	// 		$iter: expr, $static_size_hint: expr
	// 	),+
	// ) => {
	//
	// };

	// TODO: Really need this?
	// (
	// 	@collected_type
	// ) => {
	//
	// };

	(
		$(
			$iter: expr, $static_size_hint: expr $(, $global_state_access: expr)?
		),+
	) => {
		// TODO: Benchmark the best solution
		collect_iter_faster!(
			@vec
			$(
				$iter, $static_size_hint
			),+
		);
	};
}
