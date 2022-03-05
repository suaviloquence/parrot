use std::net::IpAddr;

use crate::bencode::{Data, Dictionary};

pub struct Peer {
	pub peer_id: [u8; 20],
	pub ip: IpAddr,
	pub port: u16,
}

impl Into<Data> for Peer {
	fn into(self) -> Data {
		let mut dict = Dictionary::new();
		dict.insert_str("peer id", Data::Bytes(self.peer_id.to_vec()));
		dict.insert_str(
			"ip",
			Data::Bytes(match self.ip {
				IpAddr::V4(v4) => v4.octets().to_vec(),
				IpAddr::V6(v6) => v6.octets().to_vec(),
			}),
		);
		dict.insert_str("port", Data::UInt(self.port as u64));
		Data::Dict(dict)
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
			Self::Full(peers) => peers.into_iter().map(Peer::into).collect(),
			Self::Compact(bytes) => bytes
				.into_iter()
				.map(|arr| Data::Bytes(arr.to_vec()))
				.collect(),
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

impl Into<Data> for TrackerResponse {
	fn into(self) -> Data {
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

				dict.insert_str("interval", Data::UInt(interval));

				if let Some(min_interval) = min_interval {
					dict.insert_str("min interval", Data::UInt(min_interval));
				}

				if let Some(tracker_id) = tracker_id {
					dict.insert_str("tracker id", Data::Bytes(tracker_id.into_bytes()));
				}

				dict.insert_str("complete", Data::UInt(complete));
				dict.insert_str("incomplete", Data::UInt(incomplete));

				dict.insert_str("peers", peers.into());

				if let Some(warning_message) = warning_message {
					dict.insert_str("warning message", Data::Bytes(warning_message.into_bytes()));
				}

				Data::Dict(dict)
			}
			Self::Err(s) => Data::Dict(Dictionary::from(vec![(
				"failure reason",
				Data::Bytes(s.into()),
			)])),
		}
	}
}
