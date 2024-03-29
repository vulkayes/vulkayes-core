use std::{fmt, num::NonZeroU32, ops::Deref};

use ash::vk;

use crate::prelude::{HasHandle, HostMemoryAllocator, ImageView, RenderPass, Vrc};

pub mod error;

pub struct Framebuffer {
	render_pass: Vrc<RenderPass>,
	attachments: Vec<Vrc<ImageView>>,
	framebuffer: vk::Framebuffer,
	host_memory_allocator: HostMemoryAllocator
}
impl Framebuffer {
	pub fn new(
		render_pass: Vrc<RenderPass>,
		attachments: impl Iterator<Item = Vrc<ImageView>>,
		dimensions: [NonZeroU32; 2],
		layers: NonZeroU32,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::FramebufferError> {
		let attachments = collect_iter_faster!(attachments, 8);

		#[cfg(feature = "runtime_implicit_validations")]
		{
			if !crate::util::validations::validate_all_match(
				std::iter::once(render_pass.device()).chain(attachments.iter().map(|a| a.image().device()))
			) {
				return Err(error::FramebufferError::RenderPassAttachmentsDeviceMismatch)
			}
		};

		let attachment_handles = collect_iter_faster!(
			attachments.iter().map(|a| a.handle()),
			8
		);

		let create_info = vk::FramebufferCreateInfo::builder()
			.render_pass(render_pass.handle())
			.attachments(&attachment_handles)
			.width(dimensions[0].get())
			.height(dimensions[1].get())
			.layers(layers.get());

		unsafe {
			Self::from_create_info(
				render_pass,
				attachments,
				create_info,
				host_memory_allocator
			)
		}
	}

	pub unsafe fn from_create_info(
		render_pass: Vrc<RenderPass>,
		attachments: Vec<Vrc<ImageView>>,
		create_info: impl Deref<Target = vk::FramebufferCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::FramebufferError> {
		log_trace_common!(
			"Creating framebuffer:",
			render_pass,
			create_info.deref(),
			host_memory_allocator
		);

		let framebuffer = render_pass.device().create_framebuffer(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(Framebuffer {
			render_pass,
			attachments,
			framebuffer,
			host_memory_allocator
		}))
	}

	pub const fn render_pass(&self) -> &Vrc<RenderPass> {
		&self.render_pass
	}

	pub const fn attachments(&self) -> &Vec<Vrc<ImageView>> {
		&self.attachments
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Framebuffer>, Deref, Borrow, Eq, Hash, Ord for Framebuffer {
		target = { framebuffer }
	}
}
impl Drop for Framebuffer {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.render_pass.device().destroy_framebuffer(
				self.framebuffer,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl fmt::Debug for Framebuffer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Framebuffer")
			.field("render_pass", &self.render_pass)
			.field("attachments", &self.attachments)
			.field("framebuffer", &self.safe_handle())
			.field(
				"host_memory_allocator",
				&self.host_memory_allocator
			)
			.finish()
	}
}
