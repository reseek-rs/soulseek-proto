# soulseek-proto

[![crates.io](https://img.shields.io/crates/v/soulseek-proto)](https://crates.io/crates/soulseek-proto)
[![docs.rs](https://img.shields.io/docsrs/soulseek-proto)](https://docs.rs/soulseek-proto)
[![ci](https://github.com/reseek-rs/soulseek-proto/actions/workflows/ci.yml/badge.svg)](https://github.com/reseek-rs/soulseek-proto/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/soulseek-proto)](#license)

Encoders and decoders for every message of the Soulseek wire protocol, with the frame codec, decode
limits and the data model. A client built on it is
[`soulseek-client`](https://github.com/reseek-rs/soulseek-client).

## Usage

```toml
[dependencies]
soulseek-proto = "0.3"
```

```rust
use soulseek_proto::frame::{self, FrameKind};
use soulseek_proto::server::SetListenPort;
use soulseek_proto::{DecodeError, Limits, ServerRequest};

fn main() -> Result<(), DecodeError> {
  let request = ServerRequest::SetListenPort(SetListenPort { port: 2234, obfuscation: None });
  let bytes = request.encode();

  // a socket delivers bytes in pieces: `split` takes one complete frame off the front of the
  // buffer, or returns `None` until a whole frame has arrived
  let (frame, consumed) = frame::split(FrameKind::Server, &bytes, &Limits::DEFAULT)?
    .expect("the buffer holds a whole frame");
  assert_eq!(frame.code, 2);
  assert_eq!(ServerRequest::decode(&bytes[..consumed])?, request);
  Ok(())
}
```

## Messages

| Enum                              | Connection                                               |
| --------------------------------- | -------------------------------------------------------- |
| `ServerRequest`, `ServerResponse` | Client to server, and server to client                   |
| `PeerMessage`                     | Peer (`P`) connections between two clients               |
| `DistributedMessage`              | Distributed (`D`) connections in the search tree         |
| `InitMessage`                     | The first frame on an inbound `P`, `F` or `D` connection |

- `encode` writes one complete frame, and `decode` reads exactly one
- `decode` enforces `Limits::DEFAULT`; `PeerMessage::decode_with` takes other limits
- Decoders log through `tracing`

## Features

| Feature | Default | Description                                                          |
| ------- | ------- | -------------------------------------------------------------------- |
| `serde` | Off     | `Serialize` and `Deserialize` for the message structs and data model |

- The data model includes `File`, `SearchResponse`, `UserData`, `RoomList` and `RemoteFilename`
- The message family enums, the frame types and anything that holds a password have no
  implementations

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion
in this crate, as defined in the Apache-2.0 license, is dual licensed as above, without any
additional terms or conditions.
