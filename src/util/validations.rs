/// Validates that all items in the iterator match using `Eq`.
pub fn validate_all_match<'m, M: Eq + 'm>(mut iter: impl Iterator<Item = &'m M>) -> bool {
	let first = match iter.next() {
		None => return true,
		Some(v) => v
	};

	iter.all(|m| m == first)
}
