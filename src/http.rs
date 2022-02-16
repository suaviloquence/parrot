use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct QueryString(HashMap<String, String>);

impl From<HashMap<String, String>> for QueryString {
	fn from(value: HashMap<String, String>) -> Self {
		Self(value)
	}
}

impl QueryString {
	pub fn remove(&mut self, key: &str) -> Option<String> {
		self.0.remove(key)
	}
}

fn to_hex_digit(b: u8) -> u8 {
	match b {
		b'0'..=b'9' => b - b'0',
		b'a'..=b'z' => b - b'a' + 10,
		b'A'..=b'Z' => b - b'A' + 10,
		_ => panic!("Unexpected value {} in to_hex_digit", b),
	}
}

fn url_decode(s: &str) -> String {
	let mut bytes = s.bytes();
	let mut vec = Vec::with_capacity(s.len());
	while let Some(byte) = bytes.next() {
		vec.append(&mut match byte {
			b'%' => match (bytes.next(), bytes.next()) {
				(Some(a), Some(b)) => {
					if a.is_ascii_hexdigit() && b.is_ascii_hexdigit() {
						vec![to_hex_digit(a) * 16 + to_hex_digit(b)]
					} else {
						vec![b'%', a, b]
					}
				}
				(Some(a), None) => {
					vec![b'%', a]
				}
				_ => vec![b'%'],
			},
			a => vec![a],
		})
	}
	String::from_utf8_lossy(&vec).into_owned()
}

fn url_encode(s: &str) -> String {
	let mut bytes = s.bytes();
	let mut vec = Vec::with_capacity(s.len());
	while let Some(byte) = bytes.next() {
		vec.append(&mut match byte {
			b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-' | b'.' | b'~' => vec![byte],
			_ => {
				let mut bytes: Vec<u8> = format!("{:X}", byte).bytes().collect();
				bytes.insert(0, b'%');
				bytes
			}
		})
	}
	String::from_utf8_lossy(&vec).into_owned()
}

// TODO use better error types
pub fn decode(data: &str) -> Result<QueryString, ()> {
	let mut map = HashMap::new();
	for item in data.split('&') {
		let (key, value) = match item.split_once('=') {
			Some(tup) => tup,
			None => return Err(()),
		};
		map.insert(url_decode(key), url_decode(value));
	}
	Ok(QueryString(map))
}

pub fn encode(data: QueryString) -> String {
	let mut pairs = Vec::new();
	for (k, v) in data.0.into_iter() {
		pairs.push(format!("{}={}", url_encode(&k), url_encode(&v)));
	}
	pairs.join("&")
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! qs {
		($(($k: expr, $v: expr)),*) => {{
			#[allow(unused_mut)]
			let mut map = HashMap::new();
			$(
				map.insert($k.to_string(), $v.to_string());
			)*
			QueryString::from(map)
		}};
	}
	#[test]
	fn test_url_decode() {
		assert_eq!(url_decode("h%E2%97%8Bllow"), "h○llow");
		assert_eq!(url_decode("hello%20world"), "hello world");
	}

	#[test]
	fn test_url_encode() {
		assert_eq!(url_encode("hello world"), "hello%20world");
		assert_eq!(url_encode("h○llow"), "h%E2%97%8Bllow");
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
		// hashmap is random, it could be either
		assert!(matches!(
			encode(qs!(("a", "b"), ("c", "d"))).as_str(),
			"a=b&c=d" | "c=d&a=b"
		));
		assert_eq!("w%20w=%20%20", encode(qs!(("w w", "  "))));
	}
}
