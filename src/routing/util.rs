/// Removes any '/' that may exist as the last character in a string.
pub fn stripTrailingSlash(string: &str) -> &str {
	let len = string.chars().count();
	let lastChar = string.get((len - 1)..).unwrap(); // this unwrap makes me uncomfortable
	if lastChar == "/" {
		return string.get(..(len - 1)).unwrap(); // so does this
	}

	return string;
}

pub unsafe fn extend_lifetime<'l, T>(pointer: &'l T) -> &'static T {
	::std::mem::transmute::<&'l T, &'static T>(pointer)
}