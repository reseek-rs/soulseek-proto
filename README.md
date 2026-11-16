# soulseek-proto

[![crates.io](https://img.shields.io/crates/v/soulseek-proto)](https://crates.io/crates/soulseek-proto)
[![docs.rs](https://img.shields.io/docsrs/soulseek-proto)](https://docs.rs/soulseek-proto)
[![ci](https://github.com/reseek-rs/soulseek-proto/actions/workflows/ci.yml/badge.svg)](https://github.com/reseek-rs/soulseek-proto/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/soulseek-proto)](#license)

An implementation of the Soulseek wire protocol as Rust types: every server, peer, distributed and
peer-init message with its encoder and decoder, the frame codec, decode limits and the data model.

The crate performs no I/O and has no async runtime. Bytes are decoded into messages and messages
are encoded into bytes; connection handling, login, search and transfer logic are left to the
caller. A client built on this crate is
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

## Message families

Each connection type has an enum of its messages. `encode` writes one complete frame, and
`decode` reads exactly one.

| Enum                              | Connection                                               |
| --------------------------------- | -------------------------------------------------------- |
| `ServerRequest`, `ServerResponse` | Client to server, and server to client                   |
| `PeerMessage`                     | Peer (`P`) connections between two clients               |
| `DistributedMessage`              | Distributed (`D`) connections in the search tree         |
| `InitMessage`                     | The first frame on an inbound `P`, `F` or `D` connection |

Decoders enforce the sizes in `Limits` and log through `tracing`.

## Features

`serde`, off by default, implements `Serialize` and `Deserialize` for the message structs and the
data model, such as `File`, `SearchResponse`, `UserData`, `RoomList` and `RemoteFilename`. The
message family enums, the frame types and anything that holds a password are left out.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion
in this crate, as defined in the Apache-2.0 license, is dual licensed as above, without any
additional terms or conditions.
