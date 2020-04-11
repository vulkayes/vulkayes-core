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
/// unsafe_enum_variants! {
/// 	#[derive(Debug)]
/// 	enum UnsafeEnumInner ['a] {
/// 		/// Private
/// 		Foo => { &0 },
/// 		/// Public
/// 		pub Bar => { &1 },
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
/// 	fn into(self) -> u32 {
/// 		match self.0 {
/// 			UnsafeEnumInner::Foo => { &0 },
/// 			UnsafeEnumInner::Bar => { &1 },
/// 			UnsafeEnumInner::Qux { num } => { num }
/// 		}
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! unsafe_enum_variants {
	(
		$(#[$attribute: meta])*
		enum $inner_name: ident $([ $($gen_def_tt: tt)+ ])? {
			$(
				$(#[$variant_attribute: meta])*
				$({$safety: tt})? $v: vis $variant: ident $({
					 $($variant_name: ident: $variant_type: ty),+
				})? => { $($into_code: tt)+ }
			),+
		} as pub $name: ident $([ $($gen_usage_tt: tt)+ ])? impl Into<$into_type: ty>
	) => {
		unsafe_enum_variants!(
			$(#[$attribute])*
			enum $inner_name $([ $($gen_def_tt)+ ])? {
				$(
					$(#[$variant_attribute])*
					$({$safety})? $v $variant $({ $($variant_name: $variant_type),+ })?
				),+
			} as pub $name $([ $($gen_usage_tt)+ ])?
		);
		impl $(< $($gen_def_tt)+ >)? Into<$into_type> for $name $(< $($gen_usage_tt)+ >)? {
			fn into(self) -> $into_type {
				match self.0 {
					$(
						$inner_name::$variant $({ $($variant_name),+ })? => { $($into_code)+ }
					),+
				}
			}
		}
	};

	(
		$(#[$attribute: meta])*
		enum $inner_name: ident $([ $($gen_def_tt: tt)+ ])? {
			$(
				$(#[$variant_attribute: meta])*
				$({$safety: tt})? $v: vis $variant: ident $({
					 $($variant_name: ident: $variant_type: ty),+
				})?
			),+
		} as pub $name: ident $([ $($gen_usage_tt: tt)+ ])?
	) => {
		$(#[$attribute])*
		enum $inner_name $(< $($gen_def_tt)+ >)? {
			$(
				$variant $({
					 $($variant_name: $variant_type),+
				})?
			),+
		}
		$(#[$attribute])*
		pub struct $name $(< $($gen_def_tt)+ >)? ($inner_name $(< $($gen_usage_tt)+ >)?);
		impl $(< $($gen_def_tt)+ >)? $name $(< $($gen_usage_tt)+ >)? {
			$(
				$(#[$variant_attribute])*
				#[allow(non_snake_case)]
				$v const $($safety)? fn $variant($( $( $variant_name: $variant_type ),+ )?) -> Self {
					$name(
						$inner_name::$variant $({ $($variant_name),+ })?
					)
				}
			)*
		}
	};
}

/// Generates a public enum that derives `thiserror::Error` with `VkResult` variants and their `From` impls.
///
/// Usage:
/// ```
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
				),+
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

/// Implements `Deref`, `PartialEq`, `Eq` and `Hash` for a type based on its `Deref` implementation.
///
/// Since not all types deref directly into a handle, it is possible to provide a code fragment to get handle from deref target:
/// ```
/// impl_cmmon_handle_traits! {
/// 	impl [A: Debug] Deref, PartialEq, Eq, Hash for MyType<A> {
/// 		type Target = DerefTarget { field_on_self } // Derefs to `DerefTarget` by invoking `&self.field_on_self`
///
/// 		to_handle { .handle() } // Gets a handle from `DerefTarget` by invoking `self.field_on_self.handle()`
/// 	}
/// }
/// ```
///
/// expands to:
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
		impl $([ $($impl_gen: tt)+ ])? Deref, PartialEq, Eq, Hash for $tp: ty {
			type Target = $target: ty { $($target_code: tt)+ }

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
		impl $(< $($impl_gen)+ >)? PartialEq for $tp {
			fn eq(&self, other: &Self) -> bool {
				self.$($target_code)+ $( $($to_handle_code)+ )? == other.$($target_code)+ $( $($to_handle_code)+ )?
			}
		}
		impl $(< $($impl_gen)+ >)? Eq for $tp {}
		impl $(< $($impl_gen)+ >)? std::hash::Hash for $tp {
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
/// lock_and_deref_closure!(
/// 	let foo[2]{.lock().unwrap()} => |foo_locks, foo_derefs|
/// 	let bar[0]{.lock().unwrap()} => |bar_locks: [LockGuard<Bar>; 0], bar_derefs|
/// 	{
/// 		println!("{:?} {:?}", foo_derefs, bar_derefs);
/// 	}
/// )
/// ```
///
/// expands to:
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

/// Simple enum dispatch using `Deref`. Suitable for mixed dispatch enums.
///
/// Usage:
/// ```
/// // Assuming `Trait: Deref<Target = Foo>`
/// deref_enum_dispatch! {
/// 	/// Mixed-dispatch image enum.
/// 	#[derive(Debug, Clone)]
/// 	pub enum MixedDynTrait {
/// 		Foo(Foo),
/// 		Bar(Bar),
/// 		Dyn(Box<dyn Trait>)
/// 	}: Deref<Target = Foo>
/// }
/// ```
///
/// expands to:
/// ```
/// /// Mixed-dispatch image enum.
/// #[derive(Debug, Clone)]
/// pub enum MixedDynTrait {
/// 	Foo(Foo),
/// 	Bar(Bar),
/// 	Dyn(Box<dyn Trait>)
/// }
/// impl Deref for MixedDynTrait {
/// 	type Target = Foo;
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
/// impl From<Box<dyn Trait>> for MixedDynTrait {
/// 	fn from(value: Box<dyn Trait>) -> Self {
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
