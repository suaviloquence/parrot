use std::net::{IpAddr, SocketAddrV4};

use crate::bencode::{Data, Dictionary};

#[derive(Clone, Debug)]
pub struct Peer {
	pub peer_id: [u8; 20],
	pub ip: IpAddr,
	pub port: u16,
}

impl Into<Dictionary> for Peer {
	fn into(self) -> Dictionary {
		let mut dict = Dictionary::new();
		dict.insert("peer id", self.peer_id);
		dict.insert("ip", format!("{:?}", self.ip));
		dict.insert("port", Data::UInt(self.port as u64));
		dict
	}
}

#[derive(Clone, Debug)]
pub enum Peers {
	Full(Vec<Peer>),
	/// first 4 bytes are ipv4, last 2 are port
	Compact(Vec<[u8; 6]>),
}

impl Into<Data> for Peers {
	fn into(self) -> Data {
		match self {
			Self::Full(peers) => Data::from(peers),
			Self::Compact(bytes) => bytes.into_iter().flatten().collect::<Vec<u8>>().into(),
		}
	}
}

impl Peers {
	pub fn create_compact(addrs: Vec<SocketAddrV4>) -> Self {
		let mut vec = Vec::new();
		for addr in addrs {
			let ip = addr.ip().octets();
			let port = addr.port().to_be_bytes();
			vec.push([ip[0], ip[1], ip[2], ip[3], port[0], port[1]]);
		}
		Self::Compact(vec)
	}
}

#[derive(Clone, Debug)]
pub enum TrackerResponse {
	Ok {
		interval: u64,
		min_interval: Option<u64>,
		tracker_id: Option<String>,
		complete: u64,
		incomplete: u64,
		peers: Peers,
		warning_message: Option<String>,
	},
	Err(&'static str),
}

impl Into<Dictionary> for TrackerResponse {
	fn into(self) -> Dictionary {
		match self {
			Self::Ok {
				interval,
				min_interval,
				tracker_id,
				complete,
				incomplete,
				peers,
				warning_message,
			} => {
				let mut dict = Dictionary::new();

				dict.insert("interval", interval);

				dict.insert_some("min interval", min_interval);

				dict.insert_some("tracker id", tracker_id);

				dict.insert("complete", complete);
				dict.insert("incomplete", incomplete);

				dict.insert("peers", peers);

				dict.insert_some("warning message", warning_message);
				dict
			}
			Self::Err(s) => Dictionary::from(vec![("failure reason", s)]),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{
		bencode::encode,
		bytes::assert_bytes_eq,
		tracker::{Peer, Peers, TrackerResponse},
	};
	use std::net::IpAddr;

	#[test]
	fn test_peer_into() {
		assert_bytes_eq(
			encode(Peer {
				ip: IpAddr::V4("127.0.0.1".parse().unwrap()),
				peer_id: [b'1'; 20],
				port: 16384,
			}),
			"d2:ip9:127.0.0.17:peer id20:111111111111111111114:porti16384ee",
		);

		assert_bytes_eq(encode(Peer {
			ip: IpAddr::V6("::1".parse().unwrap()),
			peer_id: [0; 20],
			port: 25565
		}), "d2:ip3:::17:peer id20:\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x004:porti25565ee")
	}

	#[test]
	fn test_trackerresponse_into() {
		assert_bytes_eq(
			encode(TrackerResponse::Ok {
				interval: 300,
				min_interval: None,
				tracker_id: None,
				complete: 1,
				incomplete: 0,
				peers: Peers::Full(vec![Peer {
					ip: IpAddr::V4("127.0.0.1".parse().unwrap()),
					peer_id: [b'1'; 20],
					port: 16384,
				}]),
				warning_message: None,
			}),
			"d8:completei1e10:incompletei0e8:intervali300e5:peersld2:ip9:127.0.0.17:peer id20:111111111111111111114:porti16384eeee"
		);
	}
}
