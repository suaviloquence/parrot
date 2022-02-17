use crate::bencode::{Data, Dictionary};

#[derive(Debug, PartialEq)]
pub struct File {
	pub length: u64,
	pub md5sum: Option<[u8; 32]>,
	pub path: Vec<Vec<u8>>,
}

impl Into<Data> for File {
	fn into(self) -> Data {
		let mut dict = Dictionary::new();
		dict.insert_str("length", Data::UInt(self.length));
		dict.insert_str(
			"path",
			Data::List(self.path.into_iter().map(|b| Data::Bytes(b)).collect()),
		);
		if let Some(md5sum) = self.md5sum {
			dict.insert_str("md5sum", Data::Bytes(Vec::from(md5sum)));
		}
		Data::Dict(dict)
	}
}

impl TryFrom<Data> for File {
	type Error = ();

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dict(mut value) = value {
			let length = match value.remove("length") {
				Some(Data::UInt(u)) => u,
				_ => return Err(()),
			};

			let md5sum = match value.remove("md5sum") {
				Some(Data::Bytes(b)) => b.as_slice().try_into().map(|c| Some(c)).map_err(|_| ())?,
				None => None,
				_ => return Err(()),
			};

			let path = match value.remove("path") {
				Some(Data::List(l)) => {
					let mut vec = Vec::new();
					for item in l {
						if let Data::Bytes(b) = item {
							vec.push(b);
						} else {
							return Err(());
						}
					}
					vec
				}
				_ => return Err(()),
			};

			Ok(Self {
				length,
				md5sum,
				path,
			})
		} else {
			Err(())
		}
	}
}

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
			try_decode_from_str("d6:lengthi40e4:pathl2:202:30ee"),
			Ok(Ok(File {
				length: 40,
				md5sum: None,
				path: vec!["20", "30"].into_iter().map(Vec::from).collect()
			}))
		);

		assert_eq!(
			try_decode_from_str(
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
		assert!(try_decode_from_str::<File>(
			"d6:lengthi25e6:md5sum31:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
		)
		.unwrap()
		.is_err());

		// missing stuff
		assert!(try_decode_from_str::<File>("de").unwrap().is_err());
	}
}
