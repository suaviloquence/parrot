use std::{collections::BTreeMap, str::Chars};

// see https://wiki.theory.org/BitTorrentSpecification#Bencoding

#[derive(Debug, PartialEq)]
pub struct DataParseError(&'static str);

#[derive(Debug)]
pub enum Data {
	/// unsigned integer type.  will always be decoded before i64
	UInt(u64),
	Int(i64),
	String(String),
	List(Vec<Data>),
	Dictionary(BTreeMap<String, Data>),
	#[doc(hidden)]
	End,
}

impl PartialEq for Data {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Int(l0), Self::Int(r0)) => l0 == r0,
			(Self::Int(a), Self::UInt(b)) | (Self::UInt(b), Self::Int(a)) => {
				match i64::try_from(*b) {
					Ok(i) => &i == a,
					Err(_) => false,
				}
			}
			(Self::UInt(l0), Self::UInt(r0)) => l0 == r0,
			(Self::String(l0), Self::String(r0)) => l0 == r0,
			(Self::List(l0), Self::List(r0)) => l0 == r0,
			(Self::Dictionary(l0), Self::Dictionary(r0)) => l0 == r0,
			_ => core::mem::discriminant(self) == core::mem::discriminant(other),
		}
	}
}

pub fn encode<T: Into<Data>>(data: T) -> String {
	let data = data.into();
	match data {
		Data::String(s) => format!("{}:{}", s.len(), s),
		Data::UInt(u) => format!("i{}e", u),
		Data::Int(i) => format!("i{}e", i),
		Data::List(list) => {
			let mut buf = String::from('l');
			for pt in list {
				buf.push_str(&encode(pt));
			}
			buf.push('e');
			buf
		}
		Data::Dictionary(dict) => {
			let mut buf = String::from('d');
			for (k, v) in dict {
				buf.push_str(&format!("{}{}", encode(Data::String(k)), encode(v)));
			}
			buf.push('e');
			buf
		}
		Data::End => panic!("Don't use Data::End to encode"),
	}
}

fn is_digit(c: &char) -> bool {
	c >= &'0' && c <= &'9'
}

pub fn decode(chars: &mut Chars) -> Result<Data, DataParseError> {
	let start = match chars.next() {
		Some(b) => b,
		None => return Err(DataParseError("Empty string.")),
	};

	if let Some(i) = start.to_digit(10) {
		let mut len: u64 = i as u64;
		while let Some(ch) = chars.next() {
			if ch == ':' {
				break;
			}
			match ch.to_digit(10) {
				Some(i) => len = len * 10 + i as u64,
				None => return Err(DataParseError("Unexpected non-number.")),
			};
		}
		let mut buf = String::new();
		for _ in 0..len {
			match chars.next() {
				Some(ch) => buf.push(ch),
				None => return Err(DataParseError("Unexpected end of data.")),
			}
		}
		return Ok(Data::String(buf));
	}

	match start {
		'e' => Ok(Data::End),
		'i' => {
			let mut buf = String::new();
			while let Some(ch) = chars.next() {
				if ch == 'e' {
					break;
				}
				if (ch == '-' && buf.len() == 0) || is_digit(&ch) {
					buf.push(ch);
				} else {
					return Err(DataParseError("Unexpected non-digit character"));
				}
			}
			if let Ok(u) = buf.parse::<u64>() {
				return Ok(Data::UInt(u));
			};
			// TODO check for -0 and leading zero which are invalid per spec
			buf.parse::<i64>()
				.map(|i| Data::Int(i))
				.map_err(|_| DataParseError("Not an integer."))
		}
		'l' => {
			let mut vec = Vec::new();

			loop {
				match decode(chars) {
					Ok(Data::End) => break,
					Ok(it) => vec.push(it),
					Err(err) => return Err(err),
				};
			}
			Ok(Data::List(vec))
		}
		'd' => {
			let mut map = BTreeMap::new();

			loop {
				let key = match decode(chars) {
					Ok(Data::End) => break,
					Ok(Data::String(k)) => k,
					Ok(_) => return Err(DataParseError("Unexpected non-key type.")),
					err => return err,
				};

				let value = match decode(chars) {
					Ok(Data::End) => return Err(DataParseError("Unexpected end of dictionary.")),
					Ok(val) => val,
					err => return err,
				};

				if map.contains_key(&key) {
					return Err(DataParseError("Key already in dictionary"));
				}
				map.insert(key, value);
			}

			Ok(Data::Dictionary(map))
		}
		_ => Err(DataParseError("Unexpected data type.")),
	}
}

pub fn try_decode_from<T: TryFrom<Data>>(
	data: &str,
) -> Result<Result<T, T::Error>, DataParseError> {
	Ok(<T as TryFrom<Data>>::try_from(decode(&mut data.chars())?))
}

#[cfg(test)]
mod tests {
	use crate::bencode::*;

	macro_rules! str {
		($x:expr) => {
			Data::String($x.to_string())
		};
	}

	macro_rules! list {
		($($x: expr),*) => {
			{
				#[allow(unused_mut)]
				let mut vec = Vec::new();
				$(
					vec.push($x);
				)*
				Data::List(vec)
			}
		};
	}

	macro_rules! dict {
		($(($x: expr, $v: expr)),*) => {
			{
				#[allow(unused_mut)]
				let mut map = BTreeMap::new();
				$(
					map.insert($x.to_string(), $v);
				)*
				Data::Dictionary(map)
			}
		};
	}

	macro_rules! assert_decode {
		($str: expr, $data: expr) => {
			assert_eq!(decode(&mut $str.chars()), Ok($data))
		};
	}

	macro_rules! assert_decode_err {
		($str: expr) => {
			assert!(decode(&mut $str.chars()).is_err())
		};
	}

	#[test]
	fn test_encode_string() {
		assert_eq!(encode(str!("spam")), "4:spam");
		assert_eq!(encode(str!("")), "0:");
	}

	#[test]
	fn test_encode_int() {
		assert_eq!(encode(Data::Int(3)), "i3e");
		assert_eq!(encode(Data::UInt(3)), "i3e");
		assert_eq!(encode(Data::Int(-3)), "i-3e");
		assert_eq!(encode(Data::Int(0)), "i0e");
	}

	#[test]
	fn test_encode_list() {
		assert_eq!(encode(list!(str!("spam"), str!("eggs"))), "l4:spam4:eggse");
		assert_eq!(encode(list!()), "le");
	}

	#[test]
	fn test_encode_dict() {
		assert_eq!(
			encode(dict!(("cow", str!("moo")), ("spam", str!("eggs")))),
			"d3:cow3:moo4:spam4:eggse"
		);

		assert_eq!(
			encode(dict!(("spam", list!(str!("a"), str!("b"))))),
			"d4:spaml1:a1:bee"
		);

		assert_eq!(
			encode(dict!(
				("publisher", str!("bob")),
				("publisher-webpage", str!("www.example.com")),
				("publisher.location", str!("home"))
			)),
			"d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee"
		);

		assert_eq!(encode(dict!()), "de");
	}

	#[test]
	fn test_decode_int() {
		assert_decode!("i3e", Data::Int(3));
		assert_decode!("i3e", Data::UInt(3));
		assert_decode!(&format!("i{}e", u64::MAX), Data::UInt(u64::MAX));
		assert_decode!(&format!("i{}e", i64::MAX), Data::Int(i64::MAX));
		assert_decode!(&format!("i{}e", i64::MIN), Data::Int(i64::MIN));
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
		assert_decode!("4:four", str!("four"));
		assert_decode!("0:", str!(""));

		// not enough length
		assert_decode_err!("4:123");

		// invalid length marker
		assert_decode_err!("4x:1234");
	}
	#[test]
	fn test_decode_list() {
		assert_decode!("l4:spam4:eggse", list!(str!("spam"), str!("eggs")));
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
			dict!(("cow", str!("moo")), ("spam", str!("eggs")))
		);

		assert_decode!(
			"d4:spaml1:a1:bee",
			dict!(("spam", list!(str!("a"), str!("b"))))
		);

		assert_decode!(
			"d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee",
			dict!(
				("publisher", str!("bob")),
				("publisher-webpage", str!("www.example.com")),
				("publisher.location", str!("home"))
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
