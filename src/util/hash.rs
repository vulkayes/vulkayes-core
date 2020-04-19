pub use inner::*;

#[cfg(feature = "insecure_hash")]
mod inner {
	// I found these to be fastest of { hashbrown, stdlib, fnv, fx } with a local benchmark, hopefully that's true for mostly everyone.
	pub type VHashMap<K, V> = rustc_hash::FxHashMap<K, V>;
	pub type VHashSet<V> = rustc_hash::FxHashSet<V>;
}

#[cfg(not(feature = "insecure_hash"))]
mod inner {
	pub type VHashMap<K, V> = std::collections::HashMap<K, V>;
	pub type VHashSet<V> = std::collections::HashSet<V>;
}
