use super::QueryString;

#[derive(PartialEq, Debug)]
pub enum TrackerEvent {
	/// trackerrequest must include event key
	STARTED,
	/// must be sent when shutting down gracefully
	STOPPED,
	/// must be sent to the tracker once download completes, but not if the download has already completed
	COMPLETED,
	/// normal periodic check
	REGULAR,
}

impl Into<&'static str> for TrackerEvent {
	fn into(self) -> &'static str {
		match self {
			Self::STARTED => "started",
			Self::STOPPED => "stopped",
			Self::COMPLETED => "completed",
			Self::REGULAR => "",
		}
	}
}

impl From<String> for TrackerEvent {
	fn from(value: String) -> Self {
		match &value as &str {
			"started" => Self::STARTED,
			"stopped" => Self::STOPPED,
			"completed" => Self::COMPLETED,
			_ => Self::REGULAR,
		}
	}
}

impl TryFrom<Vec<u8>> for TrackerEvent {
	type Error = std::string::FromUtf8Error;
	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		Ok(Self::from(String::from_utf8(data)?))
	}
}

#[derive(PartialEq, Debug)]
pub struct TrackerRequest {
	/// 20-byte sha1 hash of the value dict from MetaInfo
	pub info_hash: [u8; 20],
	/// 20-byte string unique to the client
	pub peer_id: [u8; 20],
	/// port client is listening on
	pub port: u16,
	/// conventionally amount of bytes uploaded (but not explicitly specified)
	pub uploaded: u64,
	/// conventionally amount of bytes downloaded (but not explicitly specified)
	pub downloaded: u64,
	/// number of bytes left to download
	pub left: u64,
	/// accepts peers in "Compact Mode"
	/// TODO unimplemented
	pub compact: Option<bool>,
	/// requests peer list without peer ids, lesser precedence than compact
	pub no_peer_id: Option<bool>,
	/// if omitted, a normal request performed at regular intervals
	pub event: Option<TrackerEvent>,
	/// optional canonical ip
	pub ip: Option<Vec<u8>>,
	/// number of peers being requested, default is conventionally 50
	pub numwant: Option<u64>,
	/// tracker id required if specified in a previous announce
	pub trackerid: Option<Vec<u8>>,
}

macro_rules! parse {
	($x: expr$(, $T: ident)?) => {
		String::from_utf8($x)
			.map_err(|_| ())?
			.parse$(::<$T>)?()
			.map_err(|_| ())?
	};
}

impl TryFrom<QueryString> for TrackerRequest {
	type Error = ();

	fn try_from(mut value: QueryString) -> Result<Self, Self::Error> {
		let info_hash = match value.remove("info_hash") {
			Some(s) => s.as_slice().try_into().map_err(|_| ())?,
			None => return Err(()),
		};

		let peer_id = match value.remove("peer_id") {
			Some(s) => s.as_slice().try_into().map_err(|_| ())?,
			None => return Err(()),
		};

		let port = match value.remove("port") {
			Some(s) => parse!(s),
			None => return Err(()),
		};

		let uploaded = match value.remove("uploaded") {
			Some(s) => parse!(s),
			None => return Err(()),
		};

		let downloaded = match value.remove("downloaded") {
			Some(s) => parse!(s),
			None => return Err(()),
		};

		let left = match value.remove("left") {
			Some(s) => parse!(s),
			None => return Err(()),
		};

		// let compact = value.remove("compact").map(|s| s != vec![b'0']);
		let compact = Some(false);
		let no_peer_id = value.remove("no_peer_id").map(|s| s != vec![b'0']);
		let event = value
			.remove("event")
			.map(|s| TrackerEvent::try_from(s).ok())
			.flatten();
		let ip = value.remove("ip");
		let numwant = match value.remove("numwant") {
			Some(s) => Some(parse!(s)),
			_ => None,
		};
		let trackerid = value.remove("trackerid");

		Ok(Self {
			info_hash,
			peer_id,
			port,
			uploaded,
			downloaded,
			left,
			compact,
			no_peer_id,
			event,
			ip,
			numwant,
			trackerid,
		})
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;
	use crate::tracker::QueryString;

	#[test]
	fn test_trackerrequest_from() {
		// minimal
		assert_eq!(
			TrackerRequest::try_from(QueryString::from(
				HashMap::from([
					("info_hash", "bbbbbbbbbbbbbbbbbbbb"),
					("peer_id", "aaaaaaaaaaaaaaaaaaaa"),
					("port", "8080"),
					("uploaded", "25000"),
					("downloaded", "3000"),
					("left", "200"),
				])
				.into_iter()
				.map(|(k, v)| (k.into(), v.into()))
				.collect::<HashMap<_, _>>()
			)),
			Ok(TrackerRequest {
				compact: None,
				downloaded: 3000,
				event: None,
				info_hash: [b'b'; 20],
				ip: None,
				left: 200,
				no_peer_id: None,
				numwant: None,
				peer_id: [b'a'; 20],
				port: 8080,
				trackerid: None,
				uploaded: 25000,
			})
		);
	}
}
