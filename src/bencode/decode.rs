use super::{Data, Dictionary};

#[derive(Debug, PartialEq)]
pub struct DataParseError(&'static str);

fn to_dec_digit(byte: u8) -> Option<u8> {
	match byte {
		b'0'..=b'9' => Some(byte - b'0'),
		_ => None,
	}
}

pub fn decode(bytes: &mut impl Iterator<Item = u8>) -> Result<Data, DataParseError> {
	let start = match bytes.next() {
		Some(b) => b,
		None => return Err(DataParseError("Empty string.")),
	};

	if let Some(i) = to_dec_digit(start) {
		let mut len: u64 = i as u64;
		while let Some(byte) = bytes.next() {
			if byte == b':' {
				break;
			}
			match to_dec_digit(byte) {
				Some(i) => len = len * 10 + i as u64,
				None => return Err(DataParseError("Unexpected non-number.")),
			};
		}
		let mut buf = Vec::new();
		for _ in 0..len {
			match bytes.next() {
				Some(byte) => buf.push(byte),
				None => return Err(DataParseError("Unexpected end of data.")),
			}
		}
		return Ok(Data::Bytes(buf));
	}

	match start {
		b'e' => Ok(Data::End),
		b'i' => {
			match bytes.next() {
				Some(n @ b'0'..=b'9') => {
					let mut u = (n - b'0') as u64;
					let mut completed = false;
					while let Some(byte @ b'0'..=b'9' | byte @ b'e') = bytes.next() {
						if byte == b'e' {
							completed = true;
							break;
						}
						u = u
							.checked_mul(10)
							.map(|x| x.checked_add((byte - b'0') as u64))
							.flatten()
							.ok_or(DataParseError("Integer overflow (unsigned 64-bit)"))?;
					}
					if completed {
						Ok(Data::UInt(u))
					} else {
						return Err(DataParseError("Unexpected non-digit character."));
					}
					// TODO check for -0 and leading zero which are invalid per spec
				}
				// only use signed integers when it's necessary (i.e., when it's negative)
				Some(b'-') => {
					let mut i = match bytes.next() {
						Some(byte @ b'0'..=b'9') => -((byte - b'0') as i64),
						_ => return Err(DataParseError("Unexpected non-digit character.")),
					};

					let mut completed = false;
					while let Some(byte @ (b'0'..=b'9' | b'e')) = bytes.next() {
						if byte == b'e' {
							completed = true;
							break;
						}
						i = i
							.checked_mul(10)
							// it's negative so you subtract the numbers
							.map(|x| x.checked_sub((byte - b'0') as i64))
							.flatten()
							.ok_or(DataParseError("Integer overflow (signed 64-bit)."))?;
					}
					if completed {
						Ok(Data::Int(i))
					} else {
						Err(DataParseError("Unexpected non-digit character."))
					}
				}
				_ => Err(DataParseError("Unexpected non-digit character")),
			}
		}
		b'l' => {
			let mut vec = Vec::new();

			loop {
				match decode(bytes) {
					Ok(Data::End) => break,
					Ok(it) => vec.push(it),
					Err(err) => return Err(err),
				};
			}
			Ok(Data::List(vec))
		}
		b'd' => {
			let mut map = Dictionary::new();

			loop {
				let key = match decode(bytes) {
					Ok(Data::End) => break,
					Ok(Data::Bytes(k)) => k,
					Ok(_) => return Err(DataParseError("Unexpected non-key type.")),
					err => return err,
				};

				let value = match decode(bytes) {
					Ok(Data::End) => return Err(DataParseError("Unexpected end of dictionary.")),
					Ok(val) => val,
					err => return err,
				};

				if let Some(_) = map.insert(key, value) {
					return Err(DataParseError("Duplicate key in dictionary."));
				};
			}

			Ok(Data::Dict(map))
		}
		_ => Err(DataParseError("Unexpected data type.")),
	}
}

pub fn try_decode_from<T>(
	data: &mut impl Iterator<Item = u8>,
) -> Result<Result<T, T::Error>, DataParseError>
where
	T: TryFrom<Data>,
{
	Ok(<T as TryFrom<Data>>::try_from(decode(data)?))
}

pub fn decode_vec(data: Vec<u8>) -> Result<Data, DataParseError> {
	decode(&mut data.into_iter())
}

pub fn try_decode_from_vec<T>(data: Vec<u8>) -> Result<Result<T, T::Error>, DataParseError>
where
	T: TryFrom<Data>,
{
	try_decode_from(&mut data.into_iter())
}

pub fn try_decode_from_str<T>(data: &'static str) -> Result<Result<T, T::Error>, DataParseError>
where
	T: TryFrom<Data>,
{
	try_decode_from_vec(Vec::from(data))
}

#[cfg(test)]
mod tests {
	use crate::bencode::*;

	#[test]
	fn test_decode_int() {
		assert_decode!("i3e", Data::Int(3));
		assert_decode!("i3e", Data::UInt(3));
		assert_decode!(format!("i{}e", u64::MAX).as_bytes(), Data::UInt(u64::MAX));
		assert_decode!(format!("i{}e", i64::MAX).as_bytes(), Data::Int(i64::MAX));
		assert_decode!(format!("i{}e", i64::MIN).as_bytes(), Data::Int(i64::MIN));
		assert_decode!("i-3e", Data::Int(-3));
		assert_decode!("i0e", Data::Int(0));

		// empty
		assert_decode_err!("ie");

		// just a negative sign
		assert_decode_err!("i-e");

		// negative sign in invalid place
		assert_decode_err!("i1-e");
	}

	#[test]
	fn test_decode_str() {
		assert_decode!("4:four", bytes!("four"));
		assert_decode!("0:", bytes!(""));

		// not enough length
		assert_decode_err!("4:123");

		// invalid length marker
		assert_decode_err!("4x:1234");
	}
	#[test]
	fn test_decode_list() {
		assert_decode!("l4:spam4:eggse", list!(bytes!("spam"), bytes!("eggs")));
		assert_decode!("le", list!());

		// no ending
		assert_decode_err!("lle");

		// skip over end markers in string
		assert_decode_err!("l4:eeee");

		// invalid inner data
		assert_decode_err!("l0:a0:e");
	}

	#[test]
	fn test_decode_dict() {
		assert_decode!(
			"d3:cow3:moo4:spam4:eggse",
			dict!(("cow", bytes!("moo")), ("spam", bytes!("eggs")))
		);

		assert_decode!(
			"d4:spaml1:a1:bee",
			dict!(("spam", list!(bytes!("a"), bytes!("b"))))
		);

		assert_decode!(
			"d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee",
			dict!(
				("publisher", bytes!("bob")),
				("publisher-webpage", bytes!("www.example.com")),
				("publisher.location", bytes!("home"))
			)
		);

		assert_decode!("de", dict!());

		// no ending
		assert_decode_err!("dde");

		// duplicate keys
		assert_decode_err!("d1:a3:dup1:a3:nooe");

		// non-string as key
		assert_decode_err!("di2e3:vale");

		// invalid inner data
		assert_decode_err!("d0:a0:e");
	}
}
macro_rules! assert_decode {
	($str: expr, $data: expr) => {
		assert_eq!(decode_vec(Vec::from($str)), Ok($data))
	};
}

macro_rules! assert_decode_err {
	($str: expr) => {
		assert!(decode_vec(Vec::from($str)).is_err())
	};
}

pub(crate) use assert_decode;
pub(crate) use assert_decode_err;
