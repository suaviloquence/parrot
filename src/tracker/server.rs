use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::mpsc::Sender;
use std::thread;

use super::{Peers, TrackerRequest, TrackerResponse};
use crate::config::{Config, PeerHost};
use crate::peer::{self, Peer};
use crate::tracker::IP;
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
			let mut stream = stream?;
			match self.handle_connection(stream.local_addr()?, stream.peer_addr()?, &mut stream) {
				Ok(true) => (),
				Ok(false) => write!(&mut stream, "HTTP/1.1 400 BAD REQUEST\r\n\r\n")?,
				Err(e) => {
					eprintln!("Error handling server connection: {:?}", e);
					write!(&mut stream, "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n")?;
				}
			};
		}
		Ok(())
	}
}

impl Handler for Server {
	type Ok = bool;

	fn handle_connection(
		&self,
		local: SocketAddr,
		remote: SocketAddr,
		mut stream: impl Read + Write,
	) -> std::io::Result<Self::Ok> {
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

		let data = match String::from_utf8(data) {
			Ok(d) => d,
			Err(_) => return Ok(false),
		};

		let path = match data.strip_prefix("GET /announce?") {
			Some(p) => p,
			None => return Ok(false),
		};

		let query_string = match path.split_once(" ") {
			Some((x, _)) => x,
			_ => return Ok(false),
		};

		let tracker_request =
			match super::decode(query_string).map(|qs| TrackerRequest::try_from(qs)) {
				Ok(Ok(t_r)) => t_r,
				_ => return Ok(false),
			};
		let mut body = if self.config.info_hash == tracker_request.info_hash {
			println!("Server: {:?}", remote);

			self.sender
				.send(remote)
				.expect("Error sending message from server thread.");

			let ip = match self.config.peer_host {
				PeerHost::HOST => IP::STRING(self.config.host.clone()),
				PeerHost::IP(ip) => IP::IP(ip),
				PeerHost::INFER => IP::IP(local.ip()),
			};

			println!("Sending peer with IP {:?}", ip);

			let peers = match (&tracker_request.compact, ip) {
				(&Some(true), IP::IP(IpAddr::V4(v4))) => {
					Peers::create_compact(vec![SocketAddrV4::new(v4, self.config.peer_port)])
				}
				(_, ip) => Peers::Full(vec![super::Peer {
					peer_id: peer::peer_id(),
					ip,
					port: self.config.peer_port,
				}]),
			};

			bencode::encode(TrackerResponse::Ok {
				interval: 300,
				min_interval: None,
				tracker_id: None, // TODO
				complete: 1,
				incomplete: 0,
				peers,
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

		stream.write_all(&bytes).map(|_| true)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::mpsc;

	use super::Server;
	use crate::{config::Config, peer, test::assert_stream_eq};

	#[test]
	fn test_handle_req() {
		let (sx, rx) = mpsc::channel();
		assert_stream_eq(
			Server {
				config: Config::default(),
				sender: sx.clone(),
			},
			"GET / HTTP/1.1\r\n\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:51551",
			"HTTP/1.1 400 BAD REQUEST\r\n\r\n",
		);

		rx.try_recv().expect_err("Unexpected IP in server.");

		let mut config = Config::default();
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

		config = Config::default();
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
