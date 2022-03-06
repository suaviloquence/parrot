use std::collections::BTreeMap;

use super::Data;

#[derive(Debug, PartialEq, Clone)]
pub struct Dictionary(BTreeMap<Vec<u8>, Data>);

impl Dictionary {
	pub fn new() -> Self {
		Self(BTreeMap::new())
	}
	pub fn from(data: Vec<(&str, impl Into<Data>)>) -> Self {
		Self(
			data.into_iter()
				.map(|(k, v)| (k.into(), v.into()))
				.collect(),
		)
	}

	pub fn append(&mut self, other: &mut Self) {
		self.0.append(&mut other.0)
	}

	pub fn insert(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Data>) -> Option<Data> {
		self.0.insert(key.into(), value.into())
	}

	pub fn insert_some(
		&mut self,
		key: impl Into<Vec<u8>>,
		value: Option<impl Into<Data>>,
	) -> Option<Data> {
		if let Some(data) = value {
			self.insert(key, data)
		} else {
			None
		}
	}

	pub fn remove(&mut self, key: &str) -> Option<Data> {
		self.0.remove(key.as_bytes())
	}

	pub fn remove_as<T>(&mut self, key: &str) -> Result<T, T::Error>
	where
		T: TryFrom<Data>,
		T::Error: Default,
	{
		match self.0.remove(key.as_bytes()) {
			Some(x) => x.try_into(),
			None => Err(T::Error::default()),
		}
	}

	pub fn remove_as_opt<T>(&mut self, key: &str) -> Result<Option<T>, T::Error>
	where
		T: TryFrom<Data>,
	{
		match self.0.remove(key.as_bytes()) {
			Some(x) => x.try_into().map(|x| Some(x)),
			None => Ok(None),
		}
	}
}

impl IntoIterator for Dictionary {
	type Item = (Vec<u8>, Data);

	type IntoIter = std::collections::btree_map::IntoIter<Vec<u8>, Data>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
