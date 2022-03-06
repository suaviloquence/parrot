use super::Info;
use crate::bencode::{impl_try_from_data_dict, Data, Dictionary};

#[derive(PartialEq, Debug, Clone)]
pub struct MetaInfo {
	pub info: Info,
	pub announce: Vec<u8>,
	pub announce_list: Option<Vec<Vec<u8>>>,
	pub creation_date: Option<u64>,
	pub comment: Option<Vec<u8>>,
	pub created_by: Option<Vec<u8>>,
	pub encoding: Option<Vec<u8>>,
}

impl Into<Dictionary> for MetaInfo {
	fn into(self) -> Dictionary {
		let mut dict = Dictionary::new();

		dict.insert("info", self.info);
		dict.insert("announce", self.announce);
		dict.insert_some("announce-list", self.announce_list);
		dict.insert_some("creation date", self.creation_date);
		dict.insert_some("comment", self.comment);
		dict.insert_some("created by", self.created_by);
		dict.insert_some("encoding", self.encoding);

		dict
	}
}

impl TryFrom<Dictionary> for MetaInfo {
	type Error = ();

	fn try_from(mut value: Dictionary) -> Result<Self, Self::Error> {
		let info = value.remove_as("info")?;

		let announce = value.remove_as("announce")?;

		let announce_list = value.remove_as_opt("announce-list")?;

		let comment = value.remove_as_opt("comment")?;

		let created_by = value.remove_as_opt("created by")?;

		let creation_date = value.remove_as_opt("creation date")?;

		let encoding = value.remove_as_opt("encoding")?;

		Ok(Self {
			info,
			announce,
			announce_list,
			comment,
			created_by,
			creation_date,
			encoding,
		})
	}
}

impl_try_from_data_dict!(MetaInfo);

#[cfg(test)]
mod tests {
	use crate::bencode::*;
	use crate::metainfo::*;

	#[test]
	fn test_metainfo_into() {
		// minimal
		assert_eq!(
			encode(MetaInfo {
				info: Info {
					piece_length: 0,
					pieces: "".into(),
					private: None,
					file_info: FileInfo::Single {
						length: 2,
						md5sum: None,
						name: "file".into(),
					},
				},
				announce: "".into(),
				announce_list: None,
				comment: None,
				created_by: None,
				creation_date: None,
				encoding: None,
			}),
			b"d8:announce0:4:infod6:lengthi2e4:name4:file12:piece lengthi0e6:pieces0:ee"
		);

		// all options
		assert_eq!(encode(MetaInfo {
			info: Info {
				piece_length: 5,
				pieces: "123456".into(),
				private: Some(false),
				file_info: FileInfo::Multi {
					files: vec![],
					name: "folder".into(),
				}
			},
			announce: "no".into(),
			announce_list: Some(vec!["12345".into()]),
			comment: Some("no comment".into()),
			created_by: Some("me".into()),
			creation_date: Some(0),
			encoding: Some("utf-8".into()),
		}),
		b"d8:announce2:no13:announce-listl5:12345e7:comment10:no comment10:created by2:me13:creation datei0e8:encoding5:utf-84:infod5:filesle4:name6:folder12:piece lengthi5e6:pieces6:1234567:privatei0eee"
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
					pieces: "".into(),
					private: None,
					file_info: FileInfo::Single {
						length: 2,
						md5sum: None,
						name: "file".into(),
					},
				},
				announce: "".into(),
				announce_list: None,
				comment: None,
				created_by: None,
				creation_date: None,
				encoding: None,
			}))
		);

		assert_eq!(try_decode_from(
			"d8:announce2:no13:announce-listl5:12345e7:comment10:no comment10:created by2:me13:creation datei0e8:encoding5:utf-84:infod5:filesle4:name6:folder12:piece lengthi5e6:pieces6:1234567:privatei0eee"
			),
			Ok(Ok(MetaInfo {
				info: Info {
					piece_length: 5,
					pieces: "123456".into(),
					private: Some(false),
					file_info: FileInfo::Multi {
						files: vec![],
						name: "folder".into(),
					}
				},
				announce: "no".into(),
				announce_list: Some(vec!["12345".into()]),
				comment: Some("no comment".into()),
				created_by: Some("me".into()),
				creation_date: Some(0),
				encoding: Some("utf-8".into()),
			})));
	}
}
