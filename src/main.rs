use std::{fs, sync::mpsc, thread};

use metainfo::MetaInfo;
use peer::Peer;

mod bencode;
mod metainfo;
mod peer;
mod test;
mod tracker;

fn generate_torrent() {
	let sha1sum =
		b"\xdb\x52\x4f\x06\x65\xf0\xa6\x39\x8b\x7e\xf0\x5d\xae\xec\xa5\x2c\x38\x2e\x16\x75";
	let meta_info = MetaInfo {
		announce: "http://127.0.0.1:3000/announce".into(),
		announce_list: None,
		comment: None,
		created_by: None,
		creation_date: None,
		encoding: None,
		info: metainfo::Info {
			piece_length: 16384,
			pieces: sha1sum.to_vec(),
			private: Some(true),
			file_info: metainfo::FileInfo::Single {
				name: "file.txt".into(),
				length: 19,
				md5sum: None,
			},
		},
	};

	let info = metainfo::Info {
		piece_length: 16384,
		pieces: sha1sum.to_vec(),
		private: Some(true),
		file_info: metainfo::FileInfo::Single {
			name: "file.txt".into(),
			length: 19,
			md5sum: None,
		},
	};

	fs::write("file.torrent", bencode::encode(meta_info)).expect("Error writing to file: ");
	fs::write("file.torrent.info", bencode::encode(info)).expect("Error writing to info file: ");
}

fn main() {
	generate_torrent();
	let (sender, reciever) = mpsc::channel();
	let info_hash = [
		0x41, 0xb4, 0xad, 0xfd, 0x66, 0xd4, 0x56, 0xfe, 0xbb, 0xe8, 0xf5, 0x8f, 0x6b, 0xbc, 0x55,
		0xe5, 0xcb, 0xfd, 0x45, 0x92,
	];

	let server = tracker::Server {
		// 41b4adfd66d456febbe8f58f6bbc55e5cbfd4592
		info_hash,
		sender,
	};
	thread::spawn(move || server.listen().unwrap());

	for addr in reciever {
		println!("{:?}", addr);
	}
}
