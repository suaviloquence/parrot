use std::collections::BTreeMap;

use super::Data;

#[derive(Debug, PartialEq)]
pub struct Dictionary(BTreeMap<Vec<u8>, Data>);

impl Dictionary {
	pub fn new() -> Self {
		Self(BTreeMap::new())
	}
	pub fn from(mut data: Vec<(&str, Data)>) -> Self {
		Self(
			data.into_iter()
				.map(|(k, v)| (k.as_bytes().to_vec(), v))
				.collect(),
		)
	}

	pub fn insert(&mut self, key: Vec<u8>, value: Data) -> Option<Data> {
		self.0.insert(key, value)
	}

	pub fn insert_str(&mut self, key: &str, value: Data) -> Option<Data> {
		self.0.insert(key.as_bytes().to_vec(), value)
	}

	pub fn remove(&mut self, key: &str) -> Option<Data> {
		self.0.remove(key.as_bytes())
	}
}

impl IntoIterator for Dictionary {
	type Item = (Vec<u8>, Data);

	type IntoIter = std::collections::btree_map::IntoIter<Vec<u8>, Data>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}