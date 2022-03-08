pub trait BytesExt {
	fn to_hex_string(&self) -> String;
	fn to_string(&self) -> String;
}

impl BytesExt for Vec<u8> {
	fn to_hex_string(&self) -> String {
		self.iter().map(|x| format!("{:02x}", x)).collect()
	}

	fn to_string(&self) -> String {
		String::from_utf8_lossy(&self).into_owned()
	}
}

impl BytesExt for [u8] {
	fn to_hex_string(&self) -> String {
		self.iter().map(|x| format!("{:02x}", x)).collect()
	}
	fn to_string(&self) -> String {
		String::from_utf8_lossy(&self).into_owned()
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
