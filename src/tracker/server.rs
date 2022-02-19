use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};

use crate::bencode::{self, bytes, Data, Dictionary};

use super::TrackerRequest;

pub fn listen() -> std::io::Result<()> {
	let listener = TcpListener::bind("0.0.0.0:3000")?;

	for stream in listener.incoming() {
		let mut stream = stream?;
		handle_connection(stream.local_addr()?, stream.peer_addr()?, &mut stream)?;
	}
	Ok(())
}

fn handle_connection(
	local: SocketAddr,
	remote: SocketAddr,
	mut stream: &mut (impl Read + Write),
) -> std::io::Result<()> {
	let mut data = Vec::new();
	{
		let mut buf = [0; 1024];

		loop {
			match stream.read(&mut buf) {
				Ok(1024) => data.extend_from_slice(&buf[..]),
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

	let query_string = match dbg!(path.split_once(" ")) {
		Some((x, _)) => x,
		_ => return write!(&mut stream, "HTTP/1.1 400 BAD REQUEST\r\n\r\n"),
	};

	let tracker_request = match super::decode(query_string).map(|qs| TrackerRequest::try_from(qs)) {
		Ok(Ok(t_r)) => dbg!(t_r),
		_ => return write!(&mut stream, "HTTP/1.1 400 BAD REQUEST\r\n\r\n"),
	};

	dbg!((local, remote));

	let mut body = bencode::encode(Data::Dict(Dictionary::from(vec![(
		"failure reason",
		bytes!("Unimplemented!"),
	)])));

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

#[cfg(test)]
mod tests {
	use std::{
		io::{Read, Write},
		net::SocketAddr,
	};

	use crate::tracker::server::handle_connection;
	struct MockStream {
		read: Vec<u8>,
		write: Vec<u8>,
	}

	impl MockStream {
		fn create(read: Vec<u8>) -> Self {
			Self {
				read: Vec::from(read),
				write: Vec::new(),
			}
		}
	}

	impl Read for MockStream {
		fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
			let size = buf.len().min(self.read.len());
			let mut values: Vec<u8> = self.read.drain(..size).collect();
			values.resize(buf.len(), 0);
			buf.copy_from_slice(values.as_slice());
			Ok(size)
		}
	}
	impl Write for MockStream {
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
			let size = buf.len();
			self.write.extend_from_slice(buf);
			Ok(size)
		}

		fn flush(&mut self) -> std::io::Result<()> {
			Ok(())
		}
	}

	macro_rules! stream {
		($read: expr, $local: expr, $remote: expr) => {{
			let mut stream = MockStream::create($read.into());
			(
				handle_connection(
					SocketAddr::V4($local.parse().unwrap()),
					SocketAddr::V4($remote.parse().unwrap()),
					&mut stream,
				),
				stream,
			)
		}};
	}

	macro_rules! stream_eq {
		($read: expr, $local: expr, $remote: expr, $result: expr) => {
			let (err, stream) = stream!($read, $local, $remote);
			if let Err(err) = err {
				dbg!(err);
				panic!();
			}
			let eq = stream.write == Vec::from($result);
			if !eq {
				assert_eq!(
					String::from_utf8_lossy(stream.write.as_slice()),
					String::from_utf8_lossy(Vec::from($result).as_slice())
				)
			}
		};
	}

	#[test]
	fn test_handle_req() {
		stream_eq!(
			"GET / HTTP/1.1\r\n\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:51551",
			"HTTP/1.1 400 BAD REQUEST\r\n\r\n"
		);

		stream_eq!(
			"GET /announce?info_hash=12345678901234567890&peer_id=magicnumber123456789&port=25565&uploaded=4&downloaded=5&left=6 HTTP/1.1\r\n",
			"127.0.0.1:3000",
			"192.168.7.160:50000",
			"HTTP/1.1 200 OK\r\nContent-Length: 36\r\nContent-Type: text/plain\r\n\r\nd14:failure reason14:Unimplemented!e\r\n"
		);
	}
}
