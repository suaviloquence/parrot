use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc::Sender;
use std::thread;

use super::{Peers, TrackerRequest, TrackerResponse};
use crate::bencode;
use crate::peer::{self, Peer};

pub struct Server {
	pub info_hash: [u8; 20],
	pub sender: Sender<SocketAddr>,
}

impl Server {
	pub fn listen(&self) -> std::io::Result<()> {
		let listener = TcpListener::bind("0.0.0.0:3000")?;

		let peer = Peer {
			info_hash: self.info_hash.clone(),
			peer_id: peer::peer_id(),
			port: 16384,
			sender: self.sender.clone(),
		};

		thread::spawn(move || peer.listen().unwrap());

		for stream in listener.incoming() {
			let mut stream = stream?;
			self.handle_connection(stream.local_addr()?, stream.peer_addr()?, &mut stream)?;
		}
		Ok(())
	}

	fn handle_connection(
		&self,
		local: SocketAddr,
		remote: SocketAddr,
		mut stream: &mut (impl Read + Write),
	) -> std::io::Result<()> {
		let mut data = Vec::new();
		{
			const BUF_SIZE: usize = 1024;
			let mut buf = [0; BUF_SIZE];

			loop {
				match stream.read(&mut buf) {
					Ok(BUF_SIZE) => data.extend_from_slice(&buf[..]),
					Ok(i) => {
						data.extend_from_slice(&buf[..i]);
						break;
					}
					Err(_) => break,
				}
			}
		}

		let data = String::from_utf8(data).expect("UTF-8 conversion error");

		let path = match data.strip_prefix("GET /announce?") {
			Some(p) => p,
			None => return write!(&mut stream, "HTTP/1.1 400 BAD REQUEST\r\n\r\n"),
		};

		let query_string = match path.split_once(" ") {
			Some((x, _)) => x,
			_ => return write!(&mut stream, "HTTP/1.1 400 BAD REQUEST\r\n\r\n"),
		};

		let tracker_request =
			match super::decode(query_string).map(|qs| TrackerRequest::try_from(qs)) {
				Ok(Ok(t_r)) => t_r,
				_ => return write!(&mut stream, "HTTP/1.1 400 BAD REQUEST\r\n\r\n"),
			};

		if self.info_hash != tracker_request.info_hash {
			return Ok(());
		}

		self.sender
			.send(remote)
			.expect("Error sending message from server thread.");

		let mut body = bencode::encode(TrackerResponse::Ok {
			interval: 300,
			min_interval: None,
			tracker_id: None, // TODO
			complete: 1,
			incomplete: 0,
			peers: Peers::Full(vec![super::Peer {
				peer_id: peer::peer_id(),
				ip: local.ip(),
				port: 16384,
			}]),
		});

		let mut bytes = format!(
			"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n",
			body.len()
		)
		.into_bytes();

		bytes.append(&mut body);
		bytes.push(b'\r');
		bytes.push(b'\n');

		stream.write_all(&bytes)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::mpsc;

	use super::Server;
	use crate::{peer, test::assert_stream_eq};

	#[test]
	fn test_handle_req() {
		let (sx, rx) = mpsc::channel();
		assert_stream_eq!(
			Server {
				info_hash: [1; 20],
				sender: sx.clone()
			},
			"GET / HTTP/1.1\r\n\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:51551",
			"HTTP/1.1 400 BAD REQUEST\r\n\r\n"
		);

		rx.try_recv().expect_err("Unexpected IP in server.");

		assert_stream_eq!(
			Server { info_hash: [b'1'; 20], sender: sx.clone() },
			"GET /announce?info_hash=11111111111111111111&peer_id=magicnumber123456789&port=25565&uploaded=4&downloaded=5&left=6 HTTP/1.1\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:50000",
			format!("HTTP/1.1 200 OK\r\nContent-Length: 112\r\nContent-Type: text/plain\r\n\r\nd8:completei1e10:incompletei0e8:intervali300e5:peersld2:ip4:\x7f\x00\x00\x017:peer id20:{}4:porti16384eeee\r\n", String::from_utf8(peer::peer_id().to_vec()).unwrap())
		);

		assert_eq!(rx.try_recv(), Ok("192.168.7.160:50000".parse().unwrap()));
	}
}
