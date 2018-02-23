pub fn stripTrailingSlash(string: &str) -> &str {
	let len = string.chars().count();
	let lastChar = string.get((len - 1)..).unwrap(); // this unwrap makes me uncomfortable
	if lastChar == "/" {
		return string.get(..(len - 1)).unwrap() // so does this
	}

	return string

}
