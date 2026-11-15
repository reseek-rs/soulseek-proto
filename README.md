# soulseek-proto

The Soulseek wire protocol as Rust types: every server, peer, distributed and peer-init message
with its encoder and decoder, the frame codec, decode limits and the data model.

It does no I/O and has no async runtime or opinion about what a client does with a message. Bytes
go in and messages come out, and the other way round. For a client that connects, logs in, searches
and transfers, see [`soulseek-client`](https://github.com/reseek-rs/soulseek-client).

```rust
use soulseek_proto::frame::{self, FrameKind};
use soulseek_proto::server::SetListenPort;
use soulseek_proto::{DecodeError, Limits, ServerRequest};

fn main() -> Result<(), DecodeError> {
  let request = ServerRequest::SetListenPort(SetListenPort { port: 2234, obfuscation: None });
  let bytes = request.encode();

  // a socket hands over bytes in pieces: split takes one complete frame off the front, or
  // returns None until one has arrived
  let (frame, consumed) = frame::split(FrameKind::Server, &bytes, &Limits::DEFAULT)?
    .expect("the buffer holds a whole frame");
  assert_eq!(frame.code, 2);
  assert_eq!(ServerRequest::decode(&bytes[..consumed])?, request);
  Ok(())
}
```

Each message family has an enum (`ServerRequest`, `ServerResponse`, `PeerMessage`,
`DistributedMessage`, `InitMessage`) whose `encode` writes a whole frame and whose `decode` takes
one. Decoders enforce the sizes in `Limits` and log through `tracing`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion
in this crate, as defined in the Apache-2.0 license, is dual licensed as above, without any
additional terms or conditions.
