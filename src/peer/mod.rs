mod handshake;
mod peer;

pub use handshake::*;
pub use peer::Peer;

const PEER_VERSION: [u8; 4] = [0, 0, 1, 0];
