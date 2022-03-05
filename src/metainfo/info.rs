use crate::bencode::{Data, Dictionary};

use super::FileInfo;

#[derive(PartialEq, Debug, Clone)]
pub struct Info {
	pub piece_length: u64,
	pub pieces: Vec<u8>,
	pub private: Option<bool>,
	pub file_info: FileInfo,
}

impl Into<Data> for Info {
	fn into(self) -> Data {
		let mut dict = Dictionary::new();
		dict.insert_str("piece length", Data::UInt(self.piece_length));
		dict.insert_str("pieces", Data::Bytes(self.pieces));
		if let Some(private) = self.private {
			dict.insert_str("private", Data::UInt(private as u64));
		}
		if let Data::Dict(mut file_data) = self.file_info.into() {
			dict.append(&mut file_data);
		}
		Data::Dict(dict)
	}
}

impl TryFrom<Data> for Info {
	type Error = ();

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dict(mut data) = value {
			let piece_length = match data.remove("piece length") {
				Some(Data::UInt(u)) => u,
				_ => return Err(()),
			};
			let pieces = match data.remove("pieces") {
				Some(Data::Bytes(b)) => b,
				_ => return Err(()),
			};
			let private = match data.remove("private") {
				Some(Data::UInt(u)) => Some(u != 0),
				Some(_) => return Err(()),
				None => None,
			};

			let file_info = FileInfo::try_from(Data::Dict(data))?;

			Ok(Self {
				piece_length,
				pieces,
				private,
				file_info,
			})
		} else {
			Err(())
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::bencode::*;
	use crate::metainfo::*;

	#[test]
	fn test_info_into() {
		// private = true, single file
		assert_eq!(
			encode(Info {
				piece_length: 20,
				pieces: "12345678901234567890".into(),
				private: Some(true),
				file_info: FileInfo::Single {
					length: 0,
					name: "".into(),
					md5sum: None,
				},
			}),
			b"d6:lengthi0e4:name0:12:piece lengthi20e6:pieces20:123456789012345678907:privatei1ee"
		);

		assert_eq!(
			encode(Info {
				piece_length: 1,
				pieces: "12345678901234567890".into(),
				private: None,
				file_info: FileInfo::Multi {
					name: "zamn".into(),
					files: vec![
						File {
							length: 0,
							md5sum: None,
							path: vec!["sbin", "suid", "exploit"].into_iter().map(Vec::from).collect(),
						}
					]
				}
			}),
			b"d5:filesld6:lengthi0e4:pathl4:sbin4:suid7:exploiteee4:name4:zamn12:piece lengthi1e6:pieces20:12345678901234567890e"
		);
	}

	#[test]
	fn test_info_from() {
		assert_eq!(
			try_decode_from_str("d6:lengthi0e4:name0:12:piece lengthi0e6:pieces0:e"),
			Ok(Ok(Info {
				piece_length: 0,
				pieces: "".into(),
				private: None,
				file_info: FileInfo::Single {
					length: 0,
					name: "".into(),
					md5sum: None,
				},
			}))
		);

		assert_eq!(
			try_decode_from_str(
				"d5:filesld6:lengthi0e4:pathl4:sbin4:suid7:exploiteee4:name4:zamn12:piece lengthi20e6:pieces20:123456789012345678907:privatei1ee"
			),
			Ok(Ok(Info {
				piece_length: 20,
				pieces: "12345678901234567890".into(),
				private: Some(true),
				file_info: FileInfo::Multi {
					name: "zamn".into(),
					files: vec![
						File {
							length: 0,
							md5sum: None,
							path: vec!["sbin", "suid", "exploit"].into_iter().map(Vec::from).collect(),
						}
					]
				},
			}))
		);
	}
}
