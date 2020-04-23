use std::{fmt::Debug, ops::Deref};

pub use image::Image;

use crate::{prelude::Vrc, swapchain::image::SwapchainImage};

pub mod error;

pub mod image;
pub mod params;
pub mod view;

// TODO: Is this needed?
// /// Marker trait for `Deref<Target = Image>` implementing objects.
// ///
// /// This trait is used for the dynamic dispatch in the [`MixedDynImage`](enum.MixedDynImage.html) enum.
// pub trait ImageTrait: Deref<Target = Image> + std::fmt::Debug {}
// impl<T> ImageTrait for T where T: Deref<Target = Image> + std::fmt::Debug {}

deref_enum_dispatch! {
	/// Mixed-dispatch image enum.
	#[derive(Debug, Clone)]
	pub enum MixedDynImage {
		Image(Vrc<Image>),
		SwapchainImage(Vrc<SwapchainImage>)
		// Dyn(Vrc<dyn ImageTrait>)
	}: Deref<Target = Image>
}
impl MixedDynImage {
	pub fn try_image(&self) -> Option<&Vrc<Image>> {
		match self {
			MixedDynImage::Image(i) => Some(i),
			_ => None
		}
	}

	pub fn try_swapchain_image(&self) -> Option<&Vrc<SwapchainImage>> {
		match self {
			MixedDynImage::SwapchainImage(i) => Some(i),
			_ => None
		}
	}
}
