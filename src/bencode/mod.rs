mod data;
mod decode;
mod dictionary;
mod encode;

pub(crate) use data::impl_try_from_data_dict;
pub use data::Data;
pub use decode::*;
pub use dictionary::Dictionary;
pub use encode::encode;

// see https://wiki.theory.org/BitTorrentSpecification#Bencoding
