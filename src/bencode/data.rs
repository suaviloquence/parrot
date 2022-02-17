use super::Dictionary;

#[derive(Debug)]
pub enum Data {
	/// unsigned integer type.  will always be decoded before i64
	UInt(u64),
	/// signed integer type. will only be decoded when the value is negative
	Int(i64),
	/// byte string or binary data
	Bytes(Vec<u8>),
	/// list of data
	List(Vec<Data>),
	/// dictionary (associative array) of data with byte strings as keys
	Dict(Dictionary),
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
			(Self::Bytes(l0), Self::Bytes(r0)) => l0 == r0,
			(Self::List(l0), Self::List(r0)) => l0 == r0,
			(Self::Dict(l0), Self::Dict(r0)) => l0 == r0,
			_ => core::mem::discriminant(self) == core::mem::discriminant(other),
		}
	}
}

macro_rules! bytes {
	($str: expr) => {
		crate::bencode::Data::Bytes(Vec::from($str))
	};
}

macro_rules! list {
		($($item: expr),*) => {
			{
				#[allow(unused_mut)]
				let mut vec = Vec::new();
				$(
					vec.push($item);
				)*
					crate::bencode::Data::List(vec)
			}
		};
	}

macro_rules! dict {
		($(($key: expr, $val: expr)),*) => {{
			#[allow(unused_mut)]
			let mut dict = crate::bencode::Dictionary::new();
			$(
				dict.insert(Vec::from($key), $val);
			)*
			crate::bencode::Data::Dict(dict)
		}};
	}

pub(crate) use bytes;
pub(crate) use dict;
pub(crate) use list;
