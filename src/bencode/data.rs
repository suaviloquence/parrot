use super::Dictionary;

#[derive(Debug, Clone)]
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

impl From<Vec<u8>> for Data {
	fn from(d: Vec<u8>) -> Self {
		Data::Bytes(d)
	}
}

macro_rules! impl_from_u8arr {
	($($N: expr)+) => {
		$(
			impl From<[u8; $N]> for Data {
				fn from(data: [u8; $N]) -> Self {
					Data::Bytes(Vec::from(data))
				}
			}
		)+
	};
}

// 6: compacts. 20: peer id, anything with sha1 (e.g. info hash), 32: md5sum
impl_from_u8arr!(6 20 32);

impl From<&[u8]> for Data {
	fn from(data: &[u8]) -> Self {
		Data::Bytes(data.into())
	}
}

impl From<String> for Data {
	fn from(d: String) -> Self {
		Data::Bytes(d.into_bytes())
	}
}

impl From<&str> for Data {
	fn from(s: &str) -> Self {
		Data::Bytes(s.into())
	}
}

impl<T: Into<Data>> From<Vec<T>> for Data {
	fn from(d: Vec<T>) -> Self {
		Data::List(d.into_iter().map(|x| x.into()).collect())
	}
}

impl From<u64> for Data {
	fn from(u: u64) -> Self {
		Data::UInt(u)
	}
}

impl From<i64> for Data {
	fn from(i: i64) -> Self {
		Data::Int(i)
	}
}

impl<T: Into<Dictionary>> From<T> for Data {
	fn from(d: T) -> Self {
		Data::Dict(d.into())
	}
}

macro_rules! impl_try_from_data {
	($T: ident) => {
		impl TryFrom<Data> for $T {
			type Error = ();

			fn try_from(data: Data) -> Result<Self, Self::Error> {
				if let Data::Dict(dict) = data {
					Self::try_from(dict)
				} else {
					Err(())
				}
			}
		}
	};
}

pub(crate) use impl_try_from_data;
