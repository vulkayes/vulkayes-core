use std::fmt::{Debug, Formatter};
use std::ops::Deref;

use ash::vk::AllocationCallbacks;

use crate::instance::Instance;
use crate::Vrc;

pub mod error;

struct InnerSurface {
	instance: Vrc<Instance>,
	loader: ash::extensions::khr::Surface,
	surface: ash::vk::SurfaceKHR,

	allocation_callbacks: Option<AllocationCallbacks>
}
impl Drop for InnerSurface {
	fn drop(&mut self) {
		unsafe {
			self.loader.destroy_surface(self.surface, self.allocation_callbacks.as_ref());
		}
	}
}
impl Debug for InnerSurface {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Surface")
			.field("instance", &self.instance)
			.field("loader", &"<ash::extensions::khr::Surface>")
			.field("surface", &crate::util::fmt::format_handle(self.surface))
			.field("allocation_callbacks", &self.allocation_callbacks)
			.finish()
	}
}
pub struct Surface<Window> {
	window: Window,
	inner: InnerSurface
}
impl<Window> Surface<Window> {
	/// Creates a new surface from an existing `ash::vk::SurfaceKHR`.
	///
	/// ### Safety
	///
	/// `instance` must be a parent of `surface`.
	/// `surface` must be a valid surface handle for the whole lifetime of this object.
	pub unsafe fn new(
		instance: Vrc<Instance>,
		window: Window,
		surface: ash::vk::SurfaceKHR,
		allocation_callbacks: Option<AllocationCallbacks>
	) -> Self {
		let loader = ash::extensions::khr::Surface::new(
			instance.entry().deref(),
			instance.deref().deref()
		);

		let inner = InnerSurface {
			instance,
			loader,
			surface,
			allocation_callbacks
		};


		Surface {
			window,
			inner
		}
	}

	pub fn window(&self) -> &Window {
		&self.window
	}

	/// Drops `self` but returns the owned window.
	pub fn drop_without_window(self) -> Window {
		self.window
	}
}
impl<W> Deref for Surface<W> {
	type Target = ash::vk::SurfaceKHR;

	fn deref(&self) -> &Self::Target {
		&self.inner.surface
	}
}
impl<W: Debug> Debug for Surface<W> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Surface")
			.field("window", &self.window)
			.field("inner", &self.inner)
			.finish()
	}
}