pub trait BytesExt {
	fn to_hex_string(&self) -> String;
}

impl BytesExt for Vec<u8> {
	fn to_hex_string(&self) -> String {
		self.iter().map(|x| format!("{:02x}", x)).collect()
	}
}

impl BytesExt for [u8] {
	fn to_hex_string(&self) -> String {
		self.iter().map(|x| format!("{:02x}", x)).collect()
	}
}
