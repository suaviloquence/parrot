use std::net::IpAddr;

use crate::bencode::{Data, Dictionary};

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

pub enum Peers {
	Full(Vec<Peer>),
	/// first 4 bytes are ipv4, last 2 are port
	Compact(Vec<[u8; 6]>),
}

impl Into<Data> for Peers {
	fn into(self) -> Data {
		Data::List(match self {
			Self::Full(peers) => peers.into_iter().map(Data::from).collect(),
			Self::Compact(bytes) => bytes.into_iter().map(Data::from).collect(),
		})
	}
}

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
	#[test]
	fn test_peer_into() {
		todo!()
	}
}
