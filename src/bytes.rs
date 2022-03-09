pub trait BytesExt {
	fn to_hex_string(&self) -> String;
	fn to_string(&self) -> String;
	fn to_alphanumeric_or_hex(&self) -> String;
}

impl BytesExt for [u8] {
	fn to_hex_string(&self) -> String {
		self.iter().map(|x| format!("{:02x}", x)).collect()
	}

	fn to_string(&self) -> String {
		String::from_utf8_lossy(&self).into_owned()
	}

	fn to_alphanumeric_or_hex(&self) -> String {
		let mut string = String::new();
		for ch in self.iter() {
			match ch {
				b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b':' | b' ' => string.push(*ch as char),
				_ => string += &format!("0x{:02x}", ch),
			}
		}
		string
	}
}

#[cfg(test)]
pub fn assert_bytes_eq<'a, 'b>(lhs: impl Into<Vec<u8>>, rhs: impl Into<Vec<u8>>) {
	let lhs = lhs.into();
	let rhs = rhs.into();

	if lhs != rhs {
		panic!(
			"Left is not equal to right:\n\t{:?}\n\t{:?}",
			lhs.to_string(),
			rhs.to_string()
		)
	}
}
