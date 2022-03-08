use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc::Sender;
use std::thread;

use super::{Peers, TrackerRequest, TrackerResponse};
use crate::config::Config;
use crate::peer::{self, Peer};
use crate::{bencode, Handler};

pub struct Server {
	pub config: Config,
	pub sender: Sender<SocketAddr>,
}

impl Server {
	pub fn listen(&self) -> std::io::Result<()> {
		let listener = TcpListener::bind(("0.0.0.0", self.config.server_port))?;

		let peer = Peer {
			config: self.config.clone(),
			peer_id: peer::peer_id(),
			sender: self.sender.clone(),
		};

		thread::spawn(move || peer.listen().unwrap());

		for stream in listener.incoming() {
			let stream = stream?;
			self.handle_connection(stream.local_addr()?, stream.peer_addr()?, stream)?;
		}
		Ok(())
	}
}

impl Handler for Server {
	fn handle_connection(
		&self,
		local: SocketAddr,
		remote: SocketAddr,
		mut stream: impl Read + Write,
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
		let mut body = if self.config.info_hash == tracker_request.info_hash {
			println!("Server: {:?}", remote);

			self.sender
				.send(remote)
				.expect("Error sending message from server thread.");

			bencode::encode(TrackerResponse::Ok {
				interval: 300,
				min_interval: None,
				tracker_id: None, // TODO
				complete: 1,
				incomplete: 0,
				peers: Peers::Full(vec![super::Peer {
					peer_id: peer::peer_id(),
					ip: local.ip(),
					port: self.config.peer_port,
				}]),
				warning_message: Some(format!("Your IP is {}", remote.ip())),
			})
		} else {
			bencode::encode(TrackerResponse::Err("Invalid info hash."))
		};

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
	use crate::{config::test_config, peer, test::assert_stream_eq};

	#[test]
	fn test_handle_req() {
		let (sx, rx) = mpsc::channel();
		assert_stream_eq(
			Server {
				config: test_config(),
				sender: sx.clone(),
			},
			"GET / HTTP/1.1\r\n\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:51551",
			"HTTP/1.1 400 BAD REQUEST\r\n\r\n",
		);

		rx.try_recv().expect_err("Unexpected IP in server.");

		let mut config = test_config();
		config.info_hash = [b'1'; 20];

		assert_stream_eq(
			Server { sender: sx.clone(),
			config
			},
			"GET /announce?info_hash=11111111111111111111&peer_id=magicnumber123456789&port=25565&uploaded=4&downloaded=5&left=6 HTTP/1.1\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:50000",
			format!("HTTP/1.1 200 OK\r\nContent-Length: 162\r\nContent-Type: text/plain\r\n\r\nd8:completei1e10:incompletei0e8:intervali300e5:peersld2:ip9:127.0.0.17:peer id20:{}4:porti16384eee15:warning message24:Your IP is 192.168.7.160e\r\n", String::from_utf8(peer::peer_id().to_vec()).unwrap())
		);
		assert_eq!(rx.try_recv(), Ok("192.168.7.160:50000".parse().unwrap()));

		config = test_config();
		config.info_hash = [b'2'; 20];
		assert_stream_eq(
			Server { sender: sx.clone(),
			config
			},
			"GET /announce?info_hash=11111111111111111111&peer_id=magicnumber123456789&port=25565&uploaded=4&downloaded=5&left=6 HTTP/1.1\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:50000",
			"HTTP/1.1 200 OK\r\nContent-Length: 40\r\nContent-Type: text/plain\r\n\r\nd14:failure reason18:Invalid info hash.e\r\n"
		);
	}
}
