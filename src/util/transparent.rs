/// Trait indicating that the implementor is `#[repr(transparent)]` other the `Target`.
///
/// This means the two have identical memory layouts and can safely be transmuted into each other.
///
/// ### Safety
///
/// `Self` must be `#[repr(transparent)]` over `Target`.
pub unsafe trait Transparent {
	type Target: Sized;

	fn transmute(self) -> Self::Target
	where
		Self: Sized
	{
		unsafe {
			let man = std::mem::ManuallyDrop::new(self);
			std::ptr::read(&man as *const _ as *const Self::Target)
		}
	}

	fn transmute_ref(&self) -> &Self::Target {
		unsafe { &*(self as *const Self as *const Self::Target) }
	}

	fn transmute_ref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *(self as *mut Self as *mut Self::Target) }
	}

	fn transmute_slice(me: &[Self]) -> &[Self::Target]
	where
		Self: Sized
	{
		unsafe { std::mem::transmute(me) }
	}

	fn transmute_slice_mut(me: &mut [Self]) -> &mut [Self::Target]
	where
		Self: Sized
	{
		unsafe { std::mem::transmute(me) }
	}
}

mod test {
	use crate::{descriptor::set::update::DescriptorSetWrite, util::transparent::Transparent};

	fn read_first_byte<T>(val: &T) -> u8 {
		unsafe { *(val as *const _ as *const u8) }
	}
	fn read_last_byte<T>(val: &T) -> u8 {
		unsafe { *((val as *const T).add(1) as *const u8).sub(1) }
	}
	// This is mainly for miri
	fn assert_first_last_byte_zero(val: &ash::vk::WriteDescriptorSetBuilder) {
		assert_eq!(read_first_byte(val), 0);
		assert_eq!(read_last_byte(val), 0);
	}

	#[test]
	fn transparent_transmute() {
		let value = unsafe { std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init() };
		let muted = Transparent::transmute(value);

		assert_first_last_byte_zero(&muted);
	}

	#[test]
	fn transparent_transmute_ref() {
		let value = unsafe { std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init() };
		let muted = Transparent::transmute_ref(&value);

		assert_first_last_byte_zero(&muted);
	}

	#[test]
	fn transparent_transmute_ref_mut() {
		let mut value =
			unsafe { std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init() };
		let muted = Transparent::transmute_ref_mut(&mut value);

		assert_first_last_byte_zero(&muted);
	}

	#[test]
	fn transparent_transmute_slice() {
		let value = unsafe {
			[
				std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init(),
				std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init()
			]
		};
		let muted = Transparent::transmute_slice(&value);

		assert_eq!(muted.len(), 2);
		assert_first_last_byte_zero(&muted[0]);
		assert_first_last_byte_zero(&muted[1]);
	}

	#[test]
	fn transparent_transmute_slice_mut() {
		let mut value = unsafe {
			[
				std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init(),
				std::mem::MaybeUninit::<DescriptorSetWrite>::zeroed().assume_init()
			]
		};
		let muted = Transparent::transmute_slice_mut(&mut value);

		assert_eq!(muted.len(), 2);
		assert_first_last_byte_zero(&muted[0]);
		assert_first_last_byte_zero(&muted[1]);
	}
}
