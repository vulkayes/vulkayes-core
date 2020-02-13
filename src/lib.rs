// Export `ash` because all other component will use it.
pub use ash;

pub type FastHashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type FastHashSet<V> = rustc_hash::FxHashSet<V>;

#[macro_use]
mod macros;

pub mod memory;

pub mod instance;
pub mod physical_device;
pub mod device;

#[cfg(test)]
mod tests {
    use super::*;
    use ash::Entry;
    use crate::memory::host::HostMemoryAllocator;
    use edwardium_logger::StdoutTarget;
    use log::Level;

    pub fn setup_testing_logger() {
        edwardium_logger::init(
            vec![
                StdoutTarget::new(
                    Level::Trace, Default::default()
                )
            ]
        );
    }

    #[cfg(feature = "rust_host_allocator")]
    #[test]
    fn create_instance() {
        setup_testing_logger();

        instance::Instance::new(
            Entry::new().unwrap(),
            instance::ApplicationInfo {
                application_name: "test",
                application_version: 0,
                engine_name: "test",
                engine_version: 0,
                api_version: ash::vk_make_version!(1, 2, 0)
            },
            None,
            None,
            HostMemoryAllocator::Rust(),
            instance::debug::DebugCallback::None()
        );
    }
}
