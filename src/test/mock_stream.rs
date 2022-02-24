use std::io::{Read, Write};

pub struct MockStream {
	pub read: Vec<u8>,
	pub write: Vec<u8>,
}

impl MockStream {
	pub fn create(read: Vec<u8>) -> Self {
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

macro_rules! assert_stream_eq {
	($handler: expr, $read: expr, $local: expr, $remote: expr, $result: expr) => {
		let mut stream = crate::test::MockStream::create($read.into());
		$handler
			.handle_connection(
				$local.parse().unwrap(),
				$remote.parse().unwrap(),
				&mut stream,
			)
			.expect("Error handling connection: ");
		if stream.write != Vec::from($result) {
			assert_eq!(
				String::from_utf8_lossy(stream.write.as_slice()),
				String::from_utf8_lossy(Vec::from($result).as_slice())
			)
		}
	};
}

pub(crate) use assert_stream_eq;
