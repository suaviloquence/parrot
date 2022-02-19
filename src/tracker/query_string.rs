use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct QueryString(HashMap<Vec<u8>, Vec<u8>>);

impl From<HashMap<Vec<u8>, Vec<u8>>> for QueryString {
	fn from(value: HashMap<Vec<u8>, Vec<u8>>) -> Self {
		Self(value)
	}
}

impl QueryString {
	pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
		self.0.remove(key.as_bytes())
	}
}

fn to_hex_digit(b: Option<u8>) -> Option<u8> {
	match b {
		Some(b @ b'0'..=b'9') => Some(b - b'0'),
		Some(b @ b'a'..=b'z') => Some(b - b'a' + 10),
		Some(b @ b'A'..=b'Z') => Some(b - b'A' + 10),
		_ => None,
	}
}

fn url_decode(s: &str) -> Result<Vec<u8>, ()> {
	let mut bytes = s.bytes();
	let mut vec = Vec::with_capacity(s.len());
	while let Some(byte) = bytes.next() {
		match byte {
			b'%' => match (to_hex_digit(bytes.next()), to_hex_digit(bytes.next())) {
				(Some(a), Some(b)) => vec.push(a * 16 + b),
				_ => return Err(()),
			},
			_ => vec.push(byte),
		};
	}
	Ok(vec)
}

fn url_encode(s: Vec<u8>) -> Vec<u8> {
	let mut bytes = s.into_iter();
	let mut vec = Vec::new();
	while let Some(byte) = bytes.next() {
		match byte {
			b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-' | b'.' | b'~' => vec.push(byte),
			_ => vec.append(&mut format!("%{:X}", byte).into_bytes()),
		}
	}
	vec
}

// TODO use better error types
pub fn decode(data: &str) -> Result<QueryString, ()> {
	let mut map = HashMap::new();
	for item in data.split('&') {
		let (key, value) = match item.split_once('=') {
			Some(tup) => tup,
			None => return Err(()),
		};
		map.insert(url_decode(key)?, url_decode(value)?);
	}
	Ok(QueryString(map))
}

pub fn encode(data: QueryString) -> Vec<u8> {
	let mut pairs = Vec::new();
	for (k, v) in data.0.into_iter() {
		pairs.append(&mut url_encode(k));
		pairs.push(b'=');
		pairs.append(&mut url_encode(v));
		pairs.push(b'&');
	}
	// remove superfluous &
	pairs.pop();
	pairs
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! qs {
		($(($k: expr, $v: expr)),*) => {{
			#[allow(unused_mut)]
			let mut map = HashMap::new();
			$(
				map.insert($k.into(), $v.into());
			)*
			QueryString::from(map)
		}};
	}
	#[test]
	fn test_url_decode() {
		assert_eq!(url_decode("h%E2%97%8Bllow").unwrap(), b"h\xE2\x97\x8Bllow");
		assert_eq!(url_decode("hello%20world").unwrap(), b"hello world");

		// unterminated byte sequence
		assert!(url_decode("20%").is_err());
	}

	#[test]
	fn test_url_encode() {
		assert_eq!(url_encode(Vec::from("hello world")), b"hello%20world");
		assert_eq!(url_encode(Vec::from("h○llow")), b"h%E2%97%8Bllow");
	}

	#[test]
	fn test_decode() {
		// standard
		assert_eq!(decode("a=b&c=d"), Ok(qs!(("a", "b"), ("c", "d"))));
		// url escape
		assert_eq!(decode("w%20w=%20%20"), Ok(qs!(("w w", "  "))));

		// no key
		assert!(decode("a").is_err());
		// no second half
		assert!(decode("a=b&").is_err());
	}

	#[test]
	fn test_encode() {
		let enc = encode(qs!(("a", "b"), ("c", "d")));

		// hashmap is random, it could be either
		assert!(enc == b"a=b&c=d" || enc == b"c=d&a=b");
		assert_eq!(&b"w%20w=%20%20"[..], encode(qs!(("w w", "  "))));
	}
}
