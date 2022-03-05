use super::Data;

pub fn encode(data: impl Into<Data>) -> Vec<u8> {
	let data = data.into();
	match data {
		Data::Bytes(mut s) => {
			let mut vec = Vec::new();
			vec.append(&mut s.len().to_string().into_bytes());
			vec.push(b':');
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
				buf.append(&mut encode(Data::Bytes(k)));
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
	use super::Data::*;
	use crate::bencode::*;

	#[test]
	fn test_encode_bytes() {
		assert_eq!(encode(bytes!("")), b"0:");
		assert_eq!(encode(bytes!("spam")), b"4:spam");
	}

	#[test]
	fn test_encode_int() {
		assert_eq!(encode(Int(3)), b"i3e");
		assert_eq!(encode(UInt(3)), b"i3e");
		assert_eq!(encode(Int(-3)), b"i-3e");
		assert_eq!(encode(Int(0)), b"i0e");
	}

	#[test]
	fn test_encode_list() {
		assert_eq!(
			encode(list!(bytes!("spam"), bytes!("eggs"))),
			b"l4:spam4:eggse"
		);
		assert_eq!(encode(List(vec![])), b"le");
	}

	#[test]
	fn test_encode_dict() {
		assert_eq!(
			encode(dict!(("cow", bytes!("moo")), ("spam", bytes!("eggs")))),
			b"d3:cow3:moo4:spam4:eggse"
		);

		assert_eq!(
			encode(dict!(("spam", list!(bytes!("a"), bytes!("b"))))),
			b"d4:spaml1:a1:bee"
		);

		assert_eq!(
			encode(dict!(
				("publisher", bytes!("bob")),
				("publisher-webpage", bytes!("www.example.com")),
				("publisher.location", bytes!("home"))
			)),
			b"d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee"
		);

		assert_eq!(encode(dict!()), b"de");
	}
}
