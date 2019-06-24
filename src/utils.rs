
/// Returns `true` if the given string is a proper Rust identifier.
pub fn is_rust_identifier(s: &str) -> bool {
	// Only ascii encoding is allowed.
	// Note: Maybe this check is superseeded by the `head` and `tail` check.
	if !s.is_ascii() {
		return false;
	}
	if let Some((&head, tail)) = s.as_bytes().split_first() {
		// Check if head and tail make up a proper Rust identifier.
		let head_ok = head == b'_' || head >= b'a' && head <= b'z' || head >= b'A' && head <= b'Z';
		let tail_ok = tail.iter().all(|&ch| {
			ch == b'_' || ch >= b'a' && ch <= b'z' || ch >= b'A' && ch <= b'Z' || ch >= b'0' && ch <= b'9'
		});
		head_ok && tail_ok
	} else {
		// String is empty and thus not a valid Rust identifier.
		false
	}
}
