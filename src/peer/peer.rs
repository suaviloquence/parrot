use std::{
	io::{self, Read, Write},
	net::{SocketAddr, TcpListener},
	sync::mpsc::Sender,
};

use crate::{config::Config, Handler};

use super::{Handshake, Protocol};

macro_rules! read_exact {
	($stream: expr, $buf: expr) => {
		if let Err(e) = $stream.read_exact(&mut $buf) {
			if e.kind() != io::ErrorKind::UnexpectedEof {
				eprintln!("Error reading from peer: {:?}", e);
			}
		};
	};
}

pub struct Peer {
	pub config: Config,
	pub peer_id: [u8; 20],
	pub sender: Sender<SocketAddr>,
}

impl Peer {
	pub fn listen(&self) -> std::io::Result<()> {
		let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], self.config.peer_port)))?;

		for stream in listener.incoming() {
			let stream = match stream {
				Ok(s) => s,
				Err(e) => {
					eprintln!("Error getting peer stream: {:?}", e);
					continue;
				}
			};
			if let Err(e) =
				self.handle_connection(stream.local_addr()?, stream.peer_addr()?, stream)
			{
				eprintln!("Error handling peer connection: {:?}", e);
			};
		}
		Ok(())
	}
}

impl Handler for Peer {
	type Ok = ();

	fn handle_connection(
		&self,
		_: SocketAddr,
		remote: SocketAddr,
		mut stream: impl Read + Write,
	) -> std::io::Result<Self::Ok> {
		let mut plen = [0; 1];
		read_exact!(stream, plen);

		let mut protocol = vec![0; plen[0] as usize];
		read_exact!(stream, protocol);

		let mut reserved = [0; 8];
		read_exact!(stream, reserved);

		let mut info_hash = [0; 20];
		read_exact!(stream, info_hash);

		if info_hash != self.config.info_hash {
			println!(
				"Dropped peer with unwanted info hash ({:#?}): {:?}",
				info_hash, remote
			);
			return Ok(());
		}

		let mut peer_id = [0; 20];
		read_exact!(stream, peer_id);

		println!("Peer: {:?}", remote);

		self.sender
			.send(remote)
			.expect("Error sending from peer thread");

		let handshake: Vec<u8> = Handshake {
			protocol: Protocol::BITTORRENT,
			reserved: [0; 8],
			info_hash,
			peer_id: self.peer_id,
		}
		.into();
		stream.write_all(&handshake)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::mpsc;

	use super::Peer;
	use crate::{config::Config, test::assert_stream_eq};

	#[test]
	fn test_handle_connection() {
		let (sx, rx) = mpsc::channel();
		let mut config = Config::default();
		config.info_hash = [1; 20];
		config.peer_port = 25565;
		assert_stream_eq(
			Peer {
				peer_id: [3; 20],
				config,
				sender: sx.clone(),
			},
			"\x13BitTorrent protocol\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02",
			"127.0.0.1:16384",
			"192.168.4.47:2000",
			"\x13BitTorrent protocol\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03"
		);
		assert_eq!(rx.try_recv(), Ok("192.168.4.47:2000".parse().unwrap()));
	}
}
