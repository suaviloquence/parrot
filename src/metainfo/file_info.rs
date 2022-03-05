use crate::bencode::{Data, Dictionary};

use super::File;

#[derive(Debug, PartialEq, Clone)]
pub enum FileInfo {
	Single {
		length: u64,
		md5sum: Option<[u8; 32]>,
		name: Vec<u8>,
	},
	Multi {
		name: Vec<u8>,
		files: Vec<File>,
	},
}

impl Into<Data> for FileInfo {
	fn into(self) -> Data {
		let mut dict = Dictionary::new();
		match self {
			Self::Single {
				length,
				md5sum,
				name,
			} => {
				dict.insert_str("length", Data::UInt(length));
				if let Some(md5sum) = md5sum {
					dict.insert_str("md5sum", Data::Bytes(Vec::from(md5sum)));
				}
				dict.insert_str("name", Data::Bytes(name));
			}
			Self::Multi { name, files } => {
				dict.insert_str("name", Data::Bytes(name));
				dict.insert_str(
					"files",
					Data::List(files.into_iter().map(|f| f.into()).collect()),
				);
			}
		};
		Data::Dict(dict)
	}
}

impl TryFrom<Data> for FileInfo {
	type Error = ();

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dict(mut data) = value {
			let name = match data.remove("name") {
				Some(Data::Bytes(b)) => b,
				_ => return Err(()),
			};

			if let Some(Data::List(l)) = data.remove("files") {
				let mut files = Vec::new();

				for file in l {
					files.push(File::try_from(file)?);
				}

				Ok(Self::Multi { name, files })
			} else {
				let length = match data.remove("length") {
					Some(Data::UInt(u)) => u,
					_ => return Err(()),
				};

				let md5sum = match data.remove("md5sum") {
					Some(Data::Bytes(b)) => b.try_into().map(|c| Some(c)).map_err(|_| ())?,
					None => None,
					_ => return Err(()),
				};

				Ok(Self::Single {
					name,
					length,
					md5sum,
				})
			}
		} else {
			Err(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::bencode::*;

	#[test]
	fn test_fileinfo_into() {
		// single without md5sum
		assert_eq!(
			encode(FileInfo::Single {
				length: 5,
				name: "cats.jpeg".into(),
				md5sum: None
			}),
			b"d6:lengthi5e4:name9:cats.jpege"
		);

		// single with md5sum
		assert_eq!(
			encode(FileInfo::Single {
				length: 0,
				name: "cows and cats".into(),
				md5sum: Some([b'5'; 32])
			}),
			b"d6:lengthi0e6:md5sum32:555555555555555555555555555555554:name13:cows and catse"
		);

		// minimal multi
		assert_eq!(
			encode(FileInfo::Multi {
				name: "mt".into(),
				files: vec![]
			}),
			b"d5:filesle4:name2:mte"
		);

		// substantial multi
		assert_eq!(
			encode(FileInfo::Multi {
				name: "hulking".into(),
				files: vec![
					// one with a md5
					File {
						length: 2,
						md5sum: Some([b'2'; 32]),
						path: vec!["one".into(), "two".into()],
					},
					// one without, and with no path
					File {
						length: 4,
						md5sum: None,
						path: vec![],
					}
				],
			}),
			b"d5:filesld6:lengthi2e6:md5sum32:222222222222222222222222222222224:pathl3:one3:twoeed6:lengthi4e4:pathleee4:name7:hulkinge"
		);
	}

	#[test]
	fn test_fileinfo_from() {
		// single without md5sum
		assert_eq!(
			try_decode_from_str("d6:lengthi5e4:name9:cats.jpege"),
			Ok(Ok(FileInfo::Single {
				length: 5,
				name: "cats.jpeg".into(),
				md5sum: None
			}))
		);

		// single with md5sum
		assert_eq!(
			try_decode_from_str(
				"d6:lengthi0e6:md5sum32:555555555555555555555555555555554:name13:cows and catse"
			),
			Ok(Ok(FileInfo::Single {
				length: 0,
				name: "cows and cats".into(),
				md5sum: Some([b'5'; 32])
			}))
		);

		// minimal multi
		assert_eq!(
			try_decode_from_str("d5:filesle4:name2:mte"),
			Ok(Ok(FileInfo::Multi {
				name: "mt".into(),
				files: vec![]
			})),
		);

		// substantial multi
		assert_eq!(
			try_decode_from_str(
				"d5:filesld6:lengthi2e6:md5sum32:222222222222222222222222222222224:pathl3:one3:twoeed6:lengthi4e4:pathleee4:name7:hulkinge"
			),
			Ok(Ok(FileInfo::Multi {
				name: "hulking".into(),
				files: vec![
					// one with a md5
					File {
						length: 2,
						md5sum: Some([b'2'; 32]),
						path: vec!["one".into(), "two".into()],
					},
					// one without, and with no path
					File {
						length: 4,
						md5sum: None,
						path: vec![],
					}
				],
			}))
		);

		// wrong md5 length
		assert!(
			try_decode_from_str::<FileInfo>("d6:lengthi0e6:md5sum0:4:name0:e")
				.unwrap()
				.is_err()
		);

		// bad files
		assert!(
			// length is string
			try_decode_from_str::<FileInfo>("d5:filesld6:length0:4:pathlee4:name8:bad pathee")
				.unwrap()
				.is_err()
		)
	}
}
