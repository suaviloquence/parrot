use super::Data;

pub fn encode(data: impl Into<Data>) -> Vec<u8> {
	let data = data.into();
	match data {
		Data::Bytes(mut s) => {
			let mut vec = format!("{}:", s.len()).into_bytes();
			vec.append(&mut s);
			vec
		}
		Data::UInt(u) => format!("i{}e", u).into_bytes(),
		Data::Int(i) => format!("i{}e", i).into_bytes(),
		Data::List(list) => {
			let mut buf = vec![b'l'];
			for pt in list {
				buf.append(&mut encode(pt));
			}
			buf.push(b'e');
			buf
		}
		Data::Dict(dict) => {
			let mut buf = vec![b'd'];
			for (k, v) in dict {
				buf.append(&mut encode(k));
				buf.append(&mut encode(v));
			}
			buf.push(b'e');
			buf
		}
		Data::End => panic!("Don't use Data::End to encode"),
	}
}

#[cfg(test)]
mod tests {
	use super::encode;
	use crate::bencode::{Data, Dictionary};

	#[test]
	fn test_encode_bytes() {
		assert_eq!(encode(""), b"0:");
		assert_eq!(encode("spam"), b"4:spam");
	}

	#[test]
	fn test_encode_int() {
		assert_eq!(encode(3 as u64), b"i3e");
		assert_eq!(encode(3 as i64), b"i3e");
		assert_eq!(encode(-3 as i64), b"i-3e");
		assert_eq!(encode(0 as i64), b"i0e");
	}

	#[test]
	fn test_encode_list() {
		assert_eq!(encode(vec!["spam", "eggs"]), b"l4:spam4:eggse");
		assert_eq!(encode(Vec::<Data>::new()), b"le");
	}

	#[test]
	fn test_encode_dict() {
		assert_eq!(
			encode(Dictionary::from(vec![("cow", "moo"), ("spam", "eggs")])),
			b"d3:cow3:moo4:spam4:eggse"
		);

		assert_eq!(
			encode(Dictionary::from(vec![("spam", vec!["a", "b"])])),
			b"d4:spaml1:a1:bee"
		);

		assert_eq!(
			encode(Dictionary::from(vec![
				("publisher", "bob"),
				("publisher-webpage", "www.example.com"),
				("publisher.location", "home")
			])),
			b"d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee"
		);

		assert_eq!(encode(Dictionary::new()), b"de");
	}
}
