use std::collections::BTreeMap;

use crate::bencode::Data;

#[derive(Debug, PartialEq)]
pub struct File {
	length: u64,
	md5sum: Option<[char; 32]>,
	path: Vec<String>,
}

impl Into<Data> for File {
	fn into(self) -> Data {
		let mut map = BTreeMap::new();
		map.insert("length", Data::UInt(self.length));
		map.insert(
			"path",
			Data::List(self.path.into_iter().map(|s| Data::String(s)).collect()),
		);
		if let Some(md5sum) = self.md5sum {
			map.insert("md5sum", Data::String(md5sum.into_iter().collect()));
		}
		Data::Dictionary(map.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
	}
}



#[derive(Debug, PartialEq)]
pub enum FileInfo {
	Single {
		length: u64,
		md5sum: Option<[char; 32]>,
		name: String,
	},
	Multi {
		name: String,
		files: Vec<File>,
	},
}

#[derive(PartialEq, Debug)]
pub struct Info {
	piece_length: u64,
	pieces: String,
	private: Option<bool>,
}

impl Into<Data> for Info {
	fn into(self) -> Data {
		let mut map = BTreeMap::new();
		map.insert("piece length".to_owned(), Data::UInt(self.piece_length));
		map.insert("pieces".to_owned(), Data::String(self.pieces));
		if let Some(private) = self.private {
			map.insert("private".to_owned(), Data::UInt(private as u64));
		}
		Data::Dictionary(map)
	}
}

#[derive(Debug, PartialEq)]
pub struct FromDataError;

impl TryFrom<Data> for Info {
	type Error = FromDataError;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dictionary(mut data) = value {
			let piece_length = match data.remove("piece length") {
				Some(Data::UInt(u)) => u,
				_ => return Err(FromDataError),
			};
			let pieces = match data.remove("pieces") {
				Some(Data::String(s)) => s,
				_ => return Err(FromDataError),
			};
			let private = match data.remove("private") {
				Some(Data::UInt(u)) => Some(u != 0),
				Some(_) => return Err(FromDataError),
				None => None,
			};

			Ok(Self {
				piece_length,
				pieces,
				private,
			})
		} else {
			Err(FromDataError)
		}
	}
}

#[derive(PartialEq, Debug)]
pub struct MetaInfo {
	info: Info,
	announce: String,
	announce_list: Option<String>,
	creation_date: Option<u64>,
	comment: Option<String>,
	created_by: Option<String>,
	encoding: Option<String>,
}

impl Into<Data> for MetaInfo {
	fn into(self) -> Data {
		let mut map = BTreeMap::new();
		map.insert("info", self.info.into());
		map.insert("announce", Data::String(self.announce));

		if let Some(announce_list) = self.announce_list {
			map.insert("announce-list", Data::String(announce_list));
		}

		if let Some(creation_date) = self.creation_date {
			map.insert("creation date", Data::UInt(creation_date));
		}

		if let Some(comment) = self.comment {
			map.insert("comment", Data::String(comment));
		}

		if let Some(created_by) = self.created_by {
			map.insert("created by", Data::String(created_by));
		}

		if let Some(encoding) = self.encoding {
			map.insert("encoding", Data::String(encoding));
		}

		Data::Dictionary(map.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
	}
}

impl TryFrom<Data> for MetaInfo {
	type Error = FromDataError;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dictionary(mut value) = value {
			let info = match value.remove("info") {
				Some(data) => Info::try_from(data)?,
				_ => return Err(FromDataError),
			};

			let announce = match value.remove("announce") {
				Some(Data::String(s)) => s,
				_ => return Err(FromDataError),
			};

			let announce_list = match value.remove("announce-list") {
				Some(Data::String(s)) => Some(s),
				None => None,
				_ => return Err(FromDataError),
			};

			let comment = match value.remove("comment") {
				Some(Data::String(s)) => Some(s),
				None => None,
				_ => return Err(FromDataError),
			};

			let created_by = match value.remove("created by") {
				Some(Data::String(s)) => Some(s),
				None => None,
				_ => return Err(FromDataError),
			};

			let creation_date = match value.remove("creation date") {
				Some(Data::UInt(u)) => Some(u),
				None => None,
				_ => return Err(FromDataError),
			};

			let encoding = match value.remove("encoding") {
				Some(Data::String(s)) => Some(s),
				None => None,
				_ => return Err(FromDataError),
			};

			Ok(Self {
				info,
				announce,
				announce_list,
				comment,
				created_by,
				creation_date,
				encoding,
			})
		} else {
			Err(FromDataError)
		}
	}
}

#[allow(unused)]
mod tests {
	use crate::bencode::*;
	use crate::metainfo::*;

	#[test]
	fn test_file_into() {
		// without md5sum
		assert_eq!(
			encode(File {
				length: 40,
				md5sum: None,
				path: vec!["20", "30"]
					.into_iter()
					.map(|s| s.to_string())
					.collect()
			}),
			"d6:lengthi40e4:pathl2:202:30ee"
		);

		// with
		assert_eq!(encode(File {
			length: 25,
			md5sum: Some(['a'; 32]),
			path: vec!["usr", "bin", "env", "rustc"].into_iter().map(&str::to_owned).collect(),
		}), "d6:lengthi25e6:md5sum32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee")
	}

	#[test]
	fn test_info_into() {
		assert_eq!(
			encode(Info {
				piece_length: 20,
				pieces: "12345678901234567890".to_owned(),
				private: Some(true),
			}),
			"d12:piece lengthi20e6:pieces20:123456789012345678907:privatei1ee"
		);

		assert_eq!(
			encode(Info {
				piece_length: 1,
				pieces: "12345678901234567890".to_owned(),
				private: None,
			}),
			"d12:piece lengthi1e6:pieces20:12345678901234567890e"
		);
	}

	#[test]
	fn test_info_from() {
		assert_eq!(
			try_decode_from("d12:piece lengthi0e6:pieces0:e"),
			Ok(Ok(Info {
				piece_length: 0,
				pieces: "".to_string(),
				private: None,
			}))
		);

		assert_eq!(
			try_decode_from("d12:piece lengthi20e6:pieces20:012345678901234567897:privatei1ee"),
			Ok(Ok(Info {
				piece_length: 20,
				pieces: "01234567890123456789".to_string(),
				private: Some(true),
			}))
		);
	}

	#[test]
	fn test_metainfo_into() {
		// minimal
		assert_eq!(
			encode(MetaInfo {
				info: Info {
					piece_length: 0,
					pieces: "".to_owned(),
					private: None
				},
				announce: "".to_owned(),
				announce_list: None,
				comment: None,
				created_by: None,
				creation_date: None,
				encoding: None,
			}),
			"d8:announce0:4:infod12:piece lengthi0e6:pieces0:ee"
		);

		// all options
		assert_eq!(encode(MetaInfo {
			info: Info { piece_length: 5, pieces: "123456".to_owned(), private: Some(false) },
			announce: "no".to_owned(),
			announce_list: Some("12345".to_owned()),
			comment: Some("no comment".to_owned()),
			created_by: Some("me".to_owned()),
			creation_date: Some(0),
			encoding: Some("utf-8".to_owned()),
		}), "d8:announce2:no13:announce-list5:123457:comment10:no comment10:created by2:me13:creation datei0e8:encoding5:utf-84:infod12:piece lengthi5e6:pieces6:1234567:privatei0eee");
	}

	#[test]
	fn test_metainfo_from() {
		// minimal
		assert_eq!(
			try_decode_from("d8:announce0:4:infod12:piece lengthi0e6:pieces0:ee"),
			Ok(Ok(MetaInfo {
				info: Info {
					piece_length: 0,
					pieces: "".to_owned(),
					private: None
				},
				announce: "".to_owned(),
				announce_list: None,
				comment: None,
				created_by: None,
				creation_date: None,
				encoding: None,
			}))
		);

		assert_eq!(try_decode_from("d8:announce2:no13:announce-list5:123457:comment10:no comment10:created by2:me13:creation datei0e8:encoding5:utf-84:infod12:piece lengthi5e6:pieces6:1234567:privatei0eee"),
			Ok(Ok(MetaInfo {
				info: Info { piece_length: 5, pieces: "123456".to_owned(), private: Some(false) },
				announce: "no".to_owned(),
				announce_list: Some("12345".to_owned()),
				comment: Some("no comment".to_owned()),
				created_by: Some("me".to_owned()),
				creation_date: Some(0),
				encoding: Some("utf-8".to_owned()),
			})));
	}
}
