/// This macro is intended to substitute for const generics when transforming input arguments to the [Queue::submit](queue/struct.Queue.html#method.submit) function.
///
/// Usage:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// const_queue_submit!(
/// 	pub fn submit_one(
/// 		&queue,
/// 		waits: [&Semaphore; 1],
/// 		stages: [vk::PipelineStageFlags; _],
/// 		buffers: [&CommandBuffer; 1],
/// 		signals: [&Semaphore; 1],
/// 		fence: Option<&Fence>
/// 	) -> Result<(), QueueSubmitError>;
/// );
/// ```
///
/// this expands to something like the [Queue::submit_one](queue/struct.Queue.html#method.submit_one)
#[macro_export]
macro_rules! const_queue_submit {
	(
		$(#[$attribute: meta])*
		pub fn $name: ident (
			&queue,
			$waits: ident: [&Semaphore; $count_waits: literal],
			stages: [vk::PipelineStageFlags; _],
			$buffers: ident: [&CommandBuffer; $count_buffers: literal],
			$signals: ident: [&Semaphore; $count_signals: literal],
			fence: Option<&Fence>
		) -> Result<(), QueueSubmitError>;
	) => {
		$(#[$attribute])*
		#[allow(unused_variables)]
		#[allow(unused_imports)]
		pub fn $name(
			queue: &$crate::queue::Queue,
			$waits: [&$crate::sync::semaphore::Semaphore; $count_waits],
			stages: [$crate::ash::vk::PipelineStageFlags; $count_waits],
			$buffers: [&$crate::command::buffer::CommandBuffer; $count_buffers],
			$signals: [&$crate::sync::semaphore::Semaphore; $count_signals],
			fence: Option<&$crate::sync::fence::Fence>
		) -> Result<(), $crate::queue::error::QueueSubmitError> {
			use $crate::queue::error::QueueSubmitError;
			use $crate::util::sync::VutexGuard;
			use $crate::ash::vk;

			#[cfg(feature = "runtime_implicit_validations")]
			{
				for stage in stages.iter() {
					if stage.is_empty() {
						return Err(QueueSubmitError::WaitStagesEmpty)
					}
				}
				{ // check that all waits, buffers and signals come from the same device
					if !$crate::util::validations::validate_all_match(
						$waits.iter().map(|w| w.device()).chain(
							$buffers.iter().map(|b| b.pool().device())
						).chain(
							$signals.iter().map(|s| s.device())
						)
					) {
						return Err(QueueSubmitError::WaitBufferSignalDeviceMismatch)
					}
				}
				for cb in $buffers.iter() {
					if cb.pool().queue_family_index() != queue.queue_family_index() {
						return Err(QueueSubmitError::QueueFamilyMismatch)
					}
				}
				if let Some(ref fence) = fence {
					if queue.device() != fence.device() {
						return Err(QueueSubmitError::QueueFenceDeviceMismatch)
					}
				}
			}

			$crate::lock_and_deref_closure!(
				let [$waits; $count_waits]{.lock().expect("vutex poisoned")} => |$waits: [VutexGuard<vk::Semaphore>; $count_waits], w|
				let [$buffers; $count_buffers]{.lock().expect("vutex poisoned")} => |$buffers: [VutexGuard<vk::CommandBuffer>; $count_buffers], b|
				let [$signals; $count_signals]{.lock().expect("vutex poisoned")} => |$signals: [VutexGuard<vk::Semaphore>; $count_signals], s|
				{
					let submit_info = vk::SubmitInfo::builder()
						.wait_semaphores(&w)
						.wait_dst_stage_mask(&stages)
						.command_buffers(&b)
						.signal_semaphores(&s)
						.build()
					;

					unsafe {
						queue.submit(
							[submit_info],
							fence
						)
					}
				}
			)
		}
	}
}

/// This macro is intended to substitute for const generics when transforming input arguments to the [Swapchain::present](swapchain/struct.Swapchain.html#method.present) function.
///
/// Usage:
/// ```
/// # #[macro_use] extern crate vulkayes_core;
/// const_queue_present!(
/// 	pub fn present_one(
/// 		&queue,
/// 		waits: [&Semaphore; 1],
/// 		images: [&SwapchainImage; 1],
/// 		result_for_all: bool
/// 	) -> QueuePresentMultipleResult<[QueuePresentResult; _]>;
/// );
/// ```
///
/// this expands to something like the [Queue::present_one](queue/struct.Queue.html#method.present_one)
#[macro_export]
macro_rules! const_queue_present {
	(
		$(#[$attribute: meta])*
		pub fn $name: ident (
			&queue,
			$waits: ident: [&Semaphore; $count_waits: literal],
			$images: ident: [&SwapchainImage; $count_images: literal],
			result_for_all: bool
		) -> QueuePresentMultipleResult<[QueuePresentResult; _]>;
	) => {
		$(#[$attribute])*
		#[allow(unused_variables)]
		#[allow(unused_imports)]
		pub fn $name(
			queue: &$crate::queue::Queue,
			$waits: [&$crate::sync::semaphore::Semaphore; $count_waits],
			$images: [&$crate::swapchain::image::SwapchainImage; $count_images],
			result_for_all: bool
		) -> $crate::queue::error::QueuePresentMultipleResult<[$crate::queue::error::QueuePresentResult; $count_images]> {
			use $crate::queue::error::{QueuePresentMultipleResult, QueuePresentResult, QueuePresentResultValue, QueuePresentError};
			use $crate::util::sync::VutexGuard;
			use $crate::ash::vk;

			#[cfg(feature = "runtime_implicit_validations")]
			{
				if $count_images == 0 {
					return QueuePresentMultipleResult::Single(
						Err(QueuePresentError::SwapchainsEmpty)
					)
				}
				if !$crate::util::validations::validate_all_match(
					$images.iter().map(|&i| i.device().instance()).chain(
						$waits.iter().map(|&w| w.device().instance())
					)
				) {
					return QueuePresentMultipleResult::Single(
						Err(QueuePresentError::SwapchainsSempahoredInstanceMismatch)
					)
				}
			}

			// Choose any swapchain, we only need it for the `present` function which uses the loader
			let any_swapchain = $images[0].swapchain();

			let indices = $crate::seq_macro::seq_expr!(
				N in 0 .. $count_images {
					[ #( $images[N].index(), )* ]
				}
			);

			$crate::lock_and_deref_closure!(
				let [$waits; $count_waits]{.lock().expect("vutex poisoned")} => |$waits: [VutexGuard<vk::Semaphore>; $count_waits], w|
				let [$images; $count_images]{.swapchain().lock().expect("vutex poisoned")} => |$images: [VutexGuard<vk::SwapchainKHR>; $count_images], s|
				{
					let present_info = vk::PresentInfoKHR::builder()
						.wait_semaphores(&w)
						.swapchains(&s)
						.image_indices(&indices)
					;

					if result_for_all {
						// This variable should be named `results` but it breaks with `seq_expr` because of macro hygiene.
						let mut $name = [vk::Result::SUCCESS; $count_images];
						let present_info = present_info.results(&mut $name);
						let _ = unsafe {
							any_swapchain.present(
								queue,
								present_info
							)
						};

						let result_values: [QueuePresentResult; $count_images] = $crate::seq_macro::seq_expr!(
							N in 0 .. $count_images {
								[
									#(
										match $name[N] {
											vk::Result::SUCCESS => Ok(QueuePresentResultValue::SUCCESS),
											vk::Result::SUBOPTIMAL_KHR => Ok(QueuePresentResultValue::SUBOPTIMAL_KHR),
											err => Err(err.into())
										},
									)*
								]
							}
						);
						QueuePresentMultipleResult::Multiple(
							result_values
						)
					} else {
						unsafe {
							any_swapchain.present(
								queue,
								present_info
							).into()
						}
					}
				}
			)
		}
	}
}
