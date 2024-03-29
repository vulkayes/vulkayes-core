use std::{fmt, ops::Deref};

use ash::vk;
use error::RenderPassError;

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Transparent, Vrc};

pub mod error;
pub mod params;

pub mod description;

pub struct RenderPass {
	device: Vrc<Device>,
	render_pass: vk::RenderPass,
	host_memory_allocator: HostMemoryAllocator
}
impl RenderPass {
	pub fn new(
		device: Vrc<Device>,
		attachments: &[params::AttachmentDescription],
		subpasses: &[params::SubpassDescription],
		dependencies: &[vk::SubpassDependency],
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, RenderPassError> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if subpasses.len() == 0 {
				return Err(RenderPassError::SubpassesEmpty)
			}

			for dependency in dependencies {
				if dependency.src_stage_mask.is_empty() {
					return Err(RenderPassError::SrcStageMaskZero)
				}
				if dependency.dst_stage_mask.is_empty() {
					return Err(RenderPassError::DstStageMaskZero)
				}
			}
		}

		let create_info = vk::RenderPassCreateInfo::builder()
			.attachments(Transparent::transmute_slice_twice(
				attachments
			))
			.subpasses(Transparent::transmute_slice_twice(
				subpasses
			))
			.dependencies(dependencies);

		unsafe {
			Self::from_create_info(
				device,
				create_info,
				host_memory_allocator
			)
		}
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateRenderPass.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::RenderPassCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, RenderPassError> {
		if log::log_enabled!(log::Level::Trace) {
			let create_info = debugize_struct!(
				create_info;
				{
					attachments: *[attachment_count] p_attachments;
					subpasses: *[subpass_count] p_subpasses;
					dependencies: *[dependency_count] p_dependencies;
				}
			);
			log_trace_common!(
				"Creating render pass:",
				device,
				create_info.attachments,
				create_info.subpasses,
				create_info.dependencies,
				create_info,
				host_memory_allocator
			);
		}

		let render_pass = device.create_render_pass(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(RenderPass {
			device,
			render_pass,
			host_memory_allocator
		}))
	}

	#[cfg(feature = "vulkan1_2")]
	pub unsafe fn from_create_info2(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::RenderPassCreateInfo2>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, RenderPassError> {
		// use ash::version::DeviceV1_2;

		log_trace_common!(
			"Creating render pass 2:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let render_pass = device.create_render_pass2(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(RenderPass {
			device,
			render_pass,
			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::RenderPass>, Deref, Borrow, Eq, Hash, Ord for RenderPass {
		target = { render_pass }
	}
}
impl Drop for RenderPass {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device.destroy_render_pass(
				self.render_pass,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl fmt::Debug for RenderPass {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("RenderPass")
			.field("device", &self.device)
			.field("render_pass", &self.safe_handle())
			.field(
				"host_memory_allocator",
				&self.host_memory_allocator
			)
			.finish()
	}
}
