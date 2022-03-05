#[cfg(test)]
mod test {
	use crate::Handler;
	use std::io::{self, Read, Write};
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
		fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
			let size = buf.len().min(self.read.len());
			let mut values: Vec<u8> = self.read.drain(..size).collect();
			values.resize(buf.len(), 0);
			buf.copy_from_slice(values.as_slice());
			Ok(size)
		}
	}
	impl Write for MockStream {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			let size = buf.len();
			self.write.extend_from_slice(buf);
			Ok(size)
		}

		fn flush(&mut self) -> io::Result<()> {
			Ok(())
		}
	}

	#[cfg(test)]
	pub fn assert_stream_eq(
		handler: impl Handler,
		read: impl Into<Vec<u8>>,
		local: &'static str,
		remote: &'static str,
		result: impl Into<Vec<u8>>,
	) {
		let mut stream = MockStream::create(read.into());
		handler
			.handle_connection(local.parse().unwrap(), remote.parse().unwrap(), &mut stream)
			.expect("Error handling connection: ");

		let result = result.into();
		if stream.write != result {
			assert_eq!(
				String::from_utf8_lossy(stream.write.as_slice()),
				String::from_utf8_lossy(result.as_slice())
			)
		}
	}
}

#[cfg(test)]
pub use test::{assert_stream_eq, MockStream};
