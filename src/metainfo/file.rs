use crate::bencode::{impl_try_from_data_dict, Data, Dictionary};

#[derive(Debug, PartialEq, Clone)]
pub struct File {
	pub length: u64,
	pub md5sum: Option<[u8; 32]>,
	pub path: Vec<Vec<u8>>,
}

impl Into<Dictionary> for File {
	fn into(self) -> Dictionary {
		let mut dict = Dictionary::new();
		dict.insert("length", self.length);
		dict.insert("path", self.path);
		dict.insert_some("md5sum", self.md5sum);
		dict
	}
}

impl TryFrom<Dictionary> for File {
	type Error = ();

	fn try_from(mut value: Dictionary) -> Result<Self, Self::Error> {
		let length = value.remove_as("length")?;

		let md5sum = value
			.remove_as_opt("md5sum")?
			.map(Vec::try_into)
			.transpose()
			.map_err(|_| ())?;

		let path = value.remove_as("path")?;

		Ok(Self {
			length,
			md5sum,
			path,
		})
	}
}

impl_try_from_data_dict!(File);

#[cfg(test)]
mod tests {
	use super::File;
	use crate::bencode::*;

	#[test]
	fn test_file_into() {
		// without md5sum
		assert_eq!(
			encode(File {
				length: 40,
				md5sum: None,
				path: vec!["20", "30"].into_iter().map(Vec::from).collect()
			}),
			b"d6:lengthi40e4:pathl2:202:30ee"
		);

		// with
		assert_eq!(encode(File {
				length: 25,
				md5sum: Some([b'a'; 32]),
				path: vec!["usr", "bin", "env", "rustc"].into_iter().map(Vec::from).collect(),
			}),
			b"d6:lengthi25e6:md5sum32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
		);
	}

	#[test]
	fn test_file_from() {
		assert_eq!(
			try_decode_from("d6:lengthi40e4:pathl2:202:30ee"),
			Ok(Ok(File {
				length: 40,
				md5sum: None,
				path: vec!["20", "30"].into_iter().map(Vec::from).collect()
			}))
		);

		assert_eq!(
			try_decode_from(
				"d6:lengthi25e6:md5sum32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
			),
			Ok(Ok(File {
				length: 25,
				md5sum: Some([b'a'; 32]),
				path: vec!["usr", "bin", "env", "rustc"]
					.into_iter()
					.map(Vec::from)
					.collect(),
			}))
		);

		// md5 of length 31 (Ok(Err(_))
		assert!(try_decode_from::<File, _>(
			"d6:lengthi25e6:md5sum31:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
		)
		.unwrap()
		.is_err());

		// missing stuff
		assert!(try_decode_from::<File, _>("de").unwrap().is_err());
	}
}
