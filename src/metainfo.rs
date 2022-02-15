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

impl TryFrom<Data> for File {
	type Error = FromDataError;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dictionary(mut value) = value {
			let length = match value.remove("length") {
				Some(Data::UInt(u)) => u,
				_ => return Err(FromDataError),
			};

			let md5sum = match value.remove("md5sum") {
				Some(Data::String(s)) => s
					.chars()
					.collect::<Vec<_>>()
					.as_slice()
					.try_into()
					.map(|c| Some(c))
					.map_err(|_| FromDataError)?,
				None => None,
				_ => return Err(FromDataError),
			};

			let path = match value.remove("path") {
				Some(Data::List(l)) => {
					let mut vec = Vec::new();
					for item in l {
						if let Data::String(s) = item {
							vec.push(s);
						} else {
							return Err(FromDataError);
						}
					}
					vec
				}
				_ => return Err(FromDataError),
			};

			Ok(Self {
				length,
				md5sum,
				path,
			})
		} else {
			Err(FromDataError)
		}
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

impl Into<Data> for FileInfo {
	fn into(self) -> Data {
		let mut map = BTreeMap::new();
		match self {
			Self::Single {
				length,
				md5sum,
				name,
			} => {
				map.insert("length", Data::UInt(length));
				if let Some(md5sum) = md5sum {
					map.insert("md5sum", Data::String(md5sum.iter().collect()));
				}
				map.insert("name", Data::String(name));
			}
			Self::Multi { name, files } => {
				map.insert("name", Data::String(name));
				map.insert(
					"files",
					Data::List(files.into_iter().map(|f| f.into()).collect()),
				);
			}
		};
		Data::Dictionary(map.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
	}
}

impl TryFrom<Data> for FileInfo {
	type Error = FromDataError;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Dictionary(mut data) = value {
			let name = match data.remove("name") {
				Some(Data::String(s)) => s,
				_ => return Err(FromDataError),
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
					_ => return Err(FromDataError),
				};

				let md5sum = match data.remove("md5sum") {
					Some(Data::String(s)) => s
						.chars()
						.collect::<Vec<_>>()
						.as_slice()
						.try_into()
						.map(|c| Some(c))
						.map_err(|_| FromDataError)?,
					None => None,
					_ => return Err(FromDataError),
				};

				Ok(Self::Single {
					name,
					length,
					md5sum,
				})
			}
		} else {
			Err(FromDataError)
		}
	}
}

#[derive(PartialEq, Debug)]
pub struct Info {
	piece_length: u64,
	pieces: String,
	private: Option<bool>,
	file_info: FileInfo,
}

impl Into<Data> for Info {
	fn into(self) -> Data {
		let mut map = BTreeMap::new();
		map.insert("piece length".to_owned(), Data::UInt(self.piece_length));
		map.insert("pieces".to_owned(), Data::String(self.pieces));
		if let Some(private) = self.private {
			map.insert("private".to_owned(), Data::UInt(private as u64));
		}
		if let Data::Dictionary(mut file_data) = self.file_info.into() {
			map.append(&mut file_data);
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

			let file_info = FileInfo::try_from(Data::Dictionary(data))?;

			Ok(Self {
				piece_length,
				pieces,
				private,
				file_info,
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

#[cfg(test)]
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
			}),
			"d6:lengthi25e6:md5sum32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
		);
	}

	#[test]
	fn test_file_from() {
		assert_eq!(
			try_decode_from("d6:lengthi40e4:pathl2:202:30ee"),
			Ok(Ok(File {
				length: 40,
				md5sum: None,
				path: vec!["20", "30"]
					.into_iter()
					.map(|s| s.to_string())
					.collect()
			}))
		);

		assert_eq!(
			try_decode_from(
				"d6:lengthi25e6:md5sum32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
			),
			Ok(Ok(File {
				length: 25,
				md5sum: Some(['a'; 32]),
				path: vec!["usr", "bin", "env", "rustc"].into_iter().map(&str::to_owned).collect(),
			}))
		);

		// md5 of length 31 (Ok(Err(_))
		assert!(try_decode_from::<File>(
			"d6:lengthi25e6:md5sum31:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4:pathl3:usr3:bin3:env5:rustcee"
		).unwrap().is_err());

		// missing stuff
		assert!(try_decode_from::<File>("de").unwrap().is_err());
	}

	#[test]
	fn test_fileinfo_into() {
		// single without md5sum
		assert_eq!(
			encode(FileInfo::Single {
				length: 5,
				name: "cats.jpeg".to_owned(),
				md5sum: None
			}),
			"d6:lengthi5e4:name9:cats.jpege"
		);

		// single with md5sum
		assert_eq!(
			encode(FileInfo::Single {
				length: 0,
				name: "cows and cats".to_owned(),
				md5sum: Some(['5'; 32])
			}),
			"d6:lengthi0e6:md5sum32:555555555555555555555555555555554:name13:cows and catse"
		);

		// minimal multi
		assert_eq!(
			encode(FileInfo::Multi {
				name: "mt".to_owned(),
				files: vec![]
			}),
			"d5:filesle4:name2:mte"
		);

		// substantial multi
		assert_eq!(
			encode(FileInfo::Multi {
				name: "hulking".to_owned(),
				files: vec![
					// one with a md5
					File {
						length: 2,
						md5sum: Some(['2'; 32]),
						path: vec!["one".to_owned(), "two".to_owned()],
					},
					// one without, and with no path
					File {
						length: 4,
						md5sum: None,
						path: vec![],
					}
				],
			}),
			"d5:filesld6:lengthi2e6:md5sum32:222222222222222222222222222222224:pathl3:one3:twoeed6:lengthi4e4:pathleee4:name7:hulkinge"
		);
	}

	#[test]
	fn test_fileinfo_from() {
		// single without md5sum
		assert_eq!(
			try_decode_from("d6:lengthi5e4:name9:cats.jpege"),
			Ok(Ok((FileInfo::Single {
				length: 5,
				name: "cats.jpeg".to_owned(),
				md5sum: None
			})))
		);

		// single with md5sum
		assert_eq!(
			try_decode_from(
				"d6:lengthi0e6:md5sum32:555555555555555555555555555555554:name13:cows and catse"
			),
			Ok(Ok((FileInfo::Single {
				length: 0,
				name: "cows and cats".to_owned(),
				md5sum: Some(['5'; 32])
			})))
		);

		// minimal multi
		assert_eq!(
			try_decode_from("d5:filesle4:name2:mte"),
			Ok(Ok((FileInfo::Multi {
				name: "mt".to_owned(),
				files: vec![]
			}))),
		);

		// substantial multi
		assert_eq!(
			try_decode_from(
				"d5:filesld6:lengthi2e6:md5sum32:222222222222222222222222222222224:pathl3:one3:twoeed6:lengthi4e4:pathleee4:name7:hulkinge"
			),
			Ok(Ok((FileInfo::Multi {
				name: "hulking".to_owned(),
				files: vec![
					// one with a md5
					File {
						length: 2,
						md5sum: Some(['2'; 32]),
						path: vec!["one".to_owned(), "two".to_owned()],
					},
					// one without, and with no path
					File {
						length: 4,
						md5sum: None,
						path: vec![],
					}
				],
			})))
		);

		// wrong md5 length
		assert!(
			try_decode_from::<FileInfo>("d6:lengthi0e6:md5sum0:4:name0:e")
				.unwrap()
				.is_err()
		);

		// bad files
		assert!(
			// length is string
			try_decode_from::<FileInfo>("d5:filesld6:length0:4:pathlee4:name8:bad pathee")
				.unwrap()
				.is_err()
		)
	}

	#[test]
	fn test_info_into() {
		// private = true, single file
		assert_eq!(
			encode(Info {
				piece_length: 20,
				pieces: "12345678901234567890".to_owned(),
				private: Some(true),
				file_info: FileInfo::Single {
					length: 0,
					name: "".to_owned(),
					md5sum: None,
				},
			}),
			"d6:lengthi0e4:name0:12:piece lengthi20e6:pieces20:123456789012345678907:privatei1ee"
		);

		assert_eq!(
			encode(Info {
				piece_length: 1,
				pieces: "12345678901234567890".to_owned(),
				private: None,
				file_info: FileInfo::Multi {
					name: "zamn".to_owned(),
					files: vec![
						File {
							length: 0,
							md5sum: None,
							path: vec!["sbin".to_owned(), "suid".to_owned(), "exploit".to_owned()],
						}
					]
				}
			}),
			"d5:filesld6:lengthi0e4:pathl4:sbin4:suid7:exploiteee4:name4:zamn12:piece lengthi1e6:pieces20:12345678901234567890e"
		);
	}

	#[test]
	fn test_info_from() {
		assert_eq!(
			try_decode_from("d6:lengthi0e4:name0:12:piece lengthi0e6:pieces0:e"),
			Ok(Ok(Info {
				piece_length: 0,
				pieces: "".to_string(),
				private: None,
				file_info: FileInfo::Single {
					length: 0,
					name: "".to_owned(),
					md5sum: None,
				},
			}))
		);

		assert_eq!(
			try_decode_from(
				"d5:filesld6:lengthi0e4:pathl4:sbin4:suid7:exploiteee4:name4:zamn12:piece lengthi20e6:pieces20:123456789012345678907:privatei1ee"
			),
			Ok(Ok(Info {
				piece_length: 20,
				pieces: "12345678901234567890".to_string(),
				private: Some(true),
				file_info: FileInfo::Multi {
					name: "zamn".to_owned(),
					files: vec![
						File {
							length: 0,
							md5sum: None,
							path: vec!["sbin".to_owned(), "suid".to_owned(), "exploit".to_owned()],
						}
					]
				},
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
					private: None,
					file_info: FileInfo::Single {
						length: 2,
						md5sum: None,
						name: "file".to_owned(),
					},
				},
				announce: "".to_owned(),
				announce_list: None,
				comment: None,
				created_by: None,
				creation_date: None,
				encoding: None,
			}),
			"d8:announce0:4:infod6:lengthi2e4:name4:file12:piece lengthi0e6:pieces0:ee"
		);

		// all options
		assert_eq!(encode(MetaInfo {
			info: Info {
				piece_length: 5,
				pieces: "123456".to_owned(),
				private: Some(false),
				file_info: FileInfo::Multi {
					files: vec![],
					name: "folder".to_owned(),
				}
			},
			announce: "no".to_owned(),
			announce_list: Some("12345".to_owned()),
			comment: Some("no comment".to_owned()),
			created_by: Some("me".to_owned()),
			creation_date: Some(0),
			encoding: Some("utf-8".to_owned()),
		}),
		"d8:announce2:no13:announce-list5:123457:comment10:no comment10:created by2:me13:creation datei0e8:encoding5:utf-84:infod5:filesle4:name6:folder12:piece lengthi5e6:pieces6:1234567:privatei0eee"
	);
	}

	#[test]
	fn test_metainfo_from() {
		// minimal
		assert_eq!(
			try_decode_from(
				"d8:announce0:4:infod6:lengthi2e4:name4:file12:piece lengthi0e6:pieces0:ee"
			),
			Ok(Ok(MetaInfo {
				info: Info {
					piece_length: 0,
					pieces: "".to_owned(),
					private: None,
					file_info: FileInfo::Single {
						length: 2,
						md5sum: None,
						name: "file".to_owned(),
					},
				},
				announce: "".to_owned(),
				announce_list: None,
				comment: None,
				created_by: None,
				creation_date: None,
				encoding: None,
			}))
		);

		assert_eq!(try_decode_from(
			"d8:announce2:no13:announce-list5:123457:comment10:no comment10:created by2:me13:creation datei0e8:encoding5:utf-84:infod5:filesle4:name6:folder12:piece lengthi5e6:pieces6:1234567:privatei0eee"
			),
			Ok(Ok(MetaInfo {
				info: Info {
					piece_length: 5,
					pieces: "123456".to_owned(),
					private: Some(false),
					file_info: FileInfo::Multi {
						files: vec![],
						name: "folder".to_owned(),
					}
				},
				announce: "no".to_owned(),
				announce_list: Some("12345".to_owned()),
				comment: Some("no comment".to_owned()),
				created_by: Some("me".to_owned()),
				creation_date: Some(0),
				encoding: Some("utf-8".to_owned()),
			})));
	}
}
