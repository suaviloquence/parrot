use std::{iter, process};

pub enum Protocol {
	BITTORRENT,
}

impl Into<&'static str> for Protocol {
	fn into(self) -> &'static str {
		match self {
			Self::BITTORRENT => "BitTorrent protocol",
		}
	}
}

pub struct Handshake {
	pub protocol: Protocol,
	pub reserved: [u8; 8],
	pub info_hash: [u8; 20],
	pub peer_id: [u8; 20],
}

impl Into<Vec<u8>> for Handshake {
	fn into(self) -> Vec<u8> {
		let mut vec = Vec::new();
		let pstr: &str = self.protocol.into();
		let mut pstr: Vec<u8> = pstr.into();

		vec.push(
			pstr.len()
				.try_into()
				.expect("Protocol length greater than 255"),
		);

		vec.append(&mut pstr);

		vec.extend_from_slice(&self.reserved[..]);
		vec.extend_from_slice(&self.info_hash[..]);
		vec.extend_from_slice(&self.peer_id[..]);

		vec
	}
}

pub fn peer_id() -> [u8; 20] {
	let mut vec = Vec::from("-PA");
	vec.extend_from_slice(&super::PEER_VERSION[..]);
	while vec.len() < 20 {
		vec.append(&mut process::id().to_string().into_bytes());
	}
	vec[..20]
		.try_into()
		.expect("Error generating version string: ")
}
