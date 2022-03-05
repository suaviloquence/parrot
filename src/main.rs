use std::{
	fs::{self, File},
	io::{self, Read, Write},
	net::SocketAddr,
	sync::mpsc,
	thread,
};

use config::Config;
use metainfo::MetaInfo;
use sha1_smol::Sha1;
use tracker::Server;

use crate::metainfo::{FileInfo, Info};

mod bencode;
mod config;
mod metainfo;
mod peer;
mod test;
mod tracker;

pub trait Handler {
	fn handle_connection(
		&self,
		local: SocketAddr,
		remote: SocketAddr,
		stream: impl Read + Write,
	) -> io::Result<()>;
}

fn generate_torrent(config: &Config) -> io::Result<[u8; 20]> {
	const PIECE_LENGTH: usize = 16384;

	let path = match &config.file {
		Some(p) => p,
		None => return Err(io::Error::new(io::ErrorKind::NotFound, "No file in config")),
	};

	let mut file = File::open(path)?;

	let mut length = 0;
	let mut pieces = Vec::new();

	loop {
		let mut piece = [0; PIECE_LENGTH];
		let len = file.read(&mut piece)?;
		if len == 0 {
			break;
		}
		length += len as u64;
		pieces.extend_from_slice(&Sha1::from(&piece[..len]).digest().bytes());
		if len < PIECE_LENGTH {
			break;
		}
	}

	let info = Info {
		piece_length: PIECE_LENGTH as u64,
		pieces,
		private: Some(true),
		file_info: FileInfo::Single {
			name: path
				.file_name()
				.ok_or(io::Error::new(
					io::ErrorKind::InvalidInput,
					"Path has no file name",
				))?
				.to_string_lossy()
				.bytes()
				.collect(),
			length,
			md5sum: None,
		},
	};

	let info_hash = Sha1::from(bencode::encode(info.clone())).digest().bytes();

	let meta_info = MetaInfo {
		announce: format!("http://{}:{}/announce", config.host, config.server_port).into_bytes(),
		announce_list: None,
		comment: None,
		created_by: None,
		creation_date: None,
		encoding: None,
		info: info.clone(),
	};

	fs::write("file.torrent", bencode::encode(meta_info))?;
	Ok(info_hash)
}

fn main() {
	let mut config = Config::load_or_exit();
	if config.file.is_some() {
		config.info_hash = generate_torrent(&config).expect("Error generating torrent.");
		println!(
			"Info Hash: {}",
			config
				.info_hash
				.iter()
				.map(|x| format!("{:x}", x))
				.collect::<Vec<_>>()
				.concat()
		)
	}
	let (sender, reciever) = mpsc::channel();

	let server = Server {
		config: config.clone(),
		sender,
	};

	thread::spawn(move || server.listen().unwrap());

	for addr in reciever {
		if let Err(e) = config.notify.run(addr.ip()) {
			eprintln!(
				"Error running {:?} with ip {}: {}",
				config.notify,
				addr.ip(),
				e
			);
		}
	}
}
