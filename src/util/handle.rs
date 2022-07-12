use std::{fmt, hash::Hash, ops::Deref};

use ash::vk;

use crate::util::{
	sync::{Vutex, VutexGuard},
	transparent::Transparent
};

/// Trait for objects that have corresponding Vulkan handles.
pub trait HasHandle<T: vk::Handle + Copy>: std::borrow::Borrow<T> + PartialEq + Eq + Hash + PartialOrd + Ord {
	fn handle(&self) -> T {
		*self.borrow()
	}

	/// Returns a safe handle borrowed from `self`.
	fn safe_handle(&self) -> SafeHandle<T> {
		unsafe { SafeHandle::from_raw(self.handle()) }
	}
}

/// Wrapper around `VutexGuard` that can be borrowed as `SafeHandle`.
pub struct VutexGuardSafeHandleBorrow<'a, T: vk::Handle> {
	guard: VutexGuard<'a, T>
}
impl<'a, T: vk::Handle + Copy> VutexGuardSafeHandleBorrow<'a, T> {
	/// ### Safety
	///
	/// `T` must be a valid handle for at least `'a`.
	pub unsafe fn from_raw(guard: VutexGuard<'a, T>) -> Self {
		Self { guard }
	}

	pub fn borrow_safe(&self) -> SafeHandle<'_, T> {
		unsafe { SafeHandle::from_raw(*self.guard) }
	}
}
impl<'a, T: vk::Handle> Deref for VutexGuardSafeHandleBorrow<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.guard.deref()
	}
}
impl<'a, T: vk::Handle> Into<VutexGuard<'a, T>> for VutexGuardSafeHandleBorrow<'a, T> {
	fn into(self) -> VutexGuard<'a, T> {
		self.guard
	}
}

/// Trait for objects that have corresponding Vulkan handles and are internally synchronized.
pub trait HasSynchronizedHandle<T: vk::Handle + Copy>: std::borrow::Borrow<Vutex<T>> + PartialEq + Eq + Hash + PartialOrd + Ord {
	fn lock_handle(&self) -> VutexGuard<T> {
		self.borrow().lock().expect("vutex poisoned")
	}

	fn lock_safe_handle(&self) -> VutexGuardSafeHandleBorrow<T> {
		unsafe { VutexGuardSafeHandleBorrow::from_raw(self.lock_handle()) }
	}
}

/// Wrapper struct around a handle that can only be safely obtained from a "smart" object and is guaranteed to be valid.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SafeHandle<'a, T: ash::vk::Handle> {
	handle: T,
	ghost: std::marker::PhantomData<&'a T>
}
impl<'a, T: ash::vk::Handle> SafeHandle<'a, T> {
	/// ### Safety
	///
	/// `handle` must be a valid handle for at least the lifetime `'a`.
	pub unsafe fn from_raw(handle: T) -> Self {
		SafeHandle { handle, ghost: std::marker::PhantomData }
	}

	pub fn into_handle(self) -> T {
		self.handle
	}
}
impl<'a, T: ash::vk::Handle + Clone> SafeHandle<'a, T> {
	/// ### Safety
	///
	/// `handle_ref` must be a reference to a valid handle for at least the lifetime `'a`.
	pub unsafe fn from_raw_reference(handle_ref: &'a T) -> Self {
		Self::from_raw(handle_ref.clone())
	}
}
impl<'a, T: ash::vk::Handle> std::ops::Deref for SafeHandle<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}
unsafe impl<'a, T: ash::vk::Handle> Transparent for SafeHandle<'a, T> {
	type Target = T;
}
impl<'a, T: ash::vk::Handle + Copy> fmt::Debug for SafeHandle<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{:?}",
			crate::util::fmt::format_handle(self.handle)
		)
	}
}
