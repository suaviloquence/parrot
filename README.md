# parrot

The pirate's canary

## Usage

`parrot -i info_hash -n notify [-f file] [-h host] [-s server_port] [-p peer_port]`

where:

- `-i` (or `--info-hash`) is the SHA1 hash of the bencoded info dictionary of the file torrent (see `-f` on how to generate this).
- `-n` (or `--notify`) is the command to run when an unexpected IP is detected. In the command, `%IP` is replaced by the unexpected IP.
- `-f` (or `--file`) optionally creates a torrent file and info hash for a given file.
- `-h` (or `--host`) sets the host of the torrent tracker (default if omitted: `127.0.0.1`)
- `-s` (or `--server-port`) sets the port the tracker listens on (default: `3000`)
- `-p` (or `--peer-port`) sets the port the peer listens on (default: `16384`)

## Glossary

- **bencode**: encoding format used by the bittorrent protocol. [[more info]](https://wiki.theory.org/BitTorrentSpecification#Bencoding)
- **info hash**: the SHA1 hash of the `info` key in the .torrent file, used to identify torrents between peers and the tracker (without revealing their contents). Can be generated with the `-f` option of parrot.
- **tracker**: the centralized server that the torrent client connects to (revealing its IP).
