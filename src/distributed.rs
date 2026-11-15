//! Distributed search messages.
//!
//! Messages on a `D` connection in the distributed search tree, which carry 1-byte codes.

use crate::codec::{Wire, family, wire_struct};
use crate::codes::DistributedCode;
use crate::error::{DecodeError, FieldError};
use crate::frame::FrameKind;
use crate::log::KeyFields;
use crate::names::is_valid_peer_username;
use crate::wire::{Reader, Writer};

/// First field of every distributed search request.
///
/// Constant, ASCII `'1'`.
pub const SEARCH_IDENTIFIER: u32 = 49;

family! {
  /// Message on a `D` connection.
  ///
  /// Sent by the client's parent or one of its children.
  pub enum DistributedMessage {
    code: DistributedCode,
    frame: Distributed,
    label: "distributed",
    // pings and searches are too frequent to log
    quiet: [Ping, SearchRequest, EmbeddedMessage],
    messages: {
      /// Deprecated.
      Ping(Ping),
      SearchRequest(SearchRequest),
      BranchLevel(BranchLevel),
      BranchRoot(BranchRoot),
      /// Obsolete.
      ChildDepth(ChildDepth),
      /// Deprecated.
      EmbeddedMessage(EmbeddedMessage),
    }
  }
}

/// Keep-alive between a parent and a child.
///
/// Distributed code 0.
///
/// Deprecated: defined as empty, though some clients reply with a `u32` token.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Ping {
  pub token: Option<u32>,
}

impl Wire for Ping {
  const MIN_BYTES: usize = 0;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      token: r.optional_fixed("token")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    if let Some(token) = self.token {
      w.u32(token);
    }
  }
}

/// Search passed down the tree.
///
/// Distributed code 3. The receiving child forwards it to its own children and answers it.
/// Written with the identifier 49 ([`SEARCH_IDENTIFIER`]). Any other identifier is rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct SearchRequest {
  pub username: String,
  pub token: u32,
  pub query: String,
}

impl Wire for SearchRequest {
  const MIN_BYTES: usize = 16;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let identifier = r.u32("identifier")?;
    if identifier != SEARCH_IDENTIFIER {
      return Err(r.fail("identifier", FieldError::BadSearchIdentifier { identifier }));
    }
    Ok(Self {
      username: r.string("username")?,
      token: r.u32("token")?,
      query: r.string("query")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    w.u32(SEARCH_IDENTIFIER);
    w.string(&self.username);
    w.u32(self.token);
    w.string(&self.query);
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.username = Some(&self.username);
    fields.token = Some(self.token);
  }
}

/// Depth of the parent in the tree.
///
/// Distributed code 4. The level is 0 when the parent is the branch root. Negative levels are
/// rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct BranchLevel {
  #[cfg_attr(test, proptest(strategy = "0..=i32::MAX as u32"))]
  pub level: u32,
}

impl Wire for BranchLevel {
  const MIN_BYTES: usize = 4;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let level = r.i32("level")?;
    let level = u32::try_from(level)
      .map_err(|_| r.fail("level", FieldError::NegativeBranchLevel { level }))?;
    Ok(Self { level })
  }

  fn write(&self, w: &mut Writer) {
    w.u32(self.level);
  }
}

/// Username at the root of the parent's branch.
///
/// Distributed code 5.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct BranchRoot {
  #[cfg_attr(test, proptest(strategy = "crate::proptests::peer_username()"))]
  pub username: String,
}

impl Wire for BranchRoot {
  const MIN_BYTES: usize = 4;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let username = r.string("username")?;
    if !is_valid_peer_username(&username) {
      return Err(r.fail("username", FieldError::InvalidUsername));
    }
    Ok(Self { username })
  }

  fn write(&self, w: &mut Writer) {
    w.string(&self.username);
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.username = Some(&self.username);
  }
}

wire_struct! {
  /// Number of levels of children below the sending child.
  ///
  /// Distributed code 7.
  ///
  /// Obsolete: parsed and ignored. The server works out the depth itself.
  pub struct ChildDepth {
    pub depth: u32,
  }
}

/// Distributed message wrapped by the server.
///
/// Server code 93. Sent by the server once the client is a branch root. Also carried by the
/// [`EmbeddedMessage`] of an old parent.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Embedded {
  /// Code of the inner distributed message.
  ///
  /// The server only ever embeds 3 ([`SearchRequest`]).
  pub distributed_code: u8,
  pub payload: Vec<u8>,
}

impl Embedded {
  /// Returns the unwrapped distributed frame.
  ///
  /// This is the frame that is broadcast to children and that [`DistributedMessage::decode`]
  /// takes.
  pub fn inner_frame(&self) -> Vec<u8> {
    let mut w = Writer::frame(FrameKind::Distributed, u32::from(self.distributed_code));
    w.bytes(&self.payload);
    w.finish()
  }
}

/// the inner code, then everything to the end of the frame
impl Wire for Embedded {
  const MIN_BYTES: usize = 1;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      distributed_code: r.u8("distributed_code")?,
      payload: r.rest().to_vec(),
    })
  }

  fn write(&self, w: &mut Writer) {
    w.u8(self.distributed_code);
    w.bytes(&self.payload);
  }
}

/// Wrapped distributed message from an old parent.
///
/// Distributed code 93. Sent by parents running a SoulseekQt version from before early 2026, or
/// an old version of another client that copied its behaviour.
///
/// Deprecated.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct EmbeddedMessage {
  #[cfg_attr(test, proptest(strategy = "crate::proptests::distributed_embedded()"))]
  pub embedded: Embedded,
  /// Whether the inner code was preceded by three zero bytes.
  ///
  /// Happens, for example, when a parent forwards the server's wrapper with its 4-byte code.
  pub padded: bool,
}

impl Wire for EmbeddedMessage {
  const MIN_BYTES: usize = 1;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let padded = r.remaining() >= 4 && r.peek(3) == [0; 3];
    if padded {
      r.bytes(3, "padding")?;
    }
    Ok(Self {
      embedded: Embedded::read(r, "embedded")?,
      padded,
    })
  }

  fn write(&self, w: &mut Writer) {
    if self.padded {
      w.bytes(&[0; 3]);
    }
    self.embedded.write(w);
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::testing::{decodes, field, p, vectors};

  fn search() -> Vec<u8> {
    p().u32(49).s("bob").u32(5).s("q").0
  }

  vectors! {
    ping: DistributedMessage::Ping(Ping { token: None }) => p().frame_u8(0);
    ping_with_token: DistributedMessage::Ping(Ping { token: Some(9) }) => p().u32(9).frame_u8(0);
    search_request:
      DistributedMessage::SearchRequest(SearchRequest { username: "bob".into(), token: 5, query: "q".into() })
      => p().raw(&search()).frame_u8(3);
    branch_level: DistributedMessage::BranchLevel(BranchLevel { level: 3 }) => p().u32(3).frame_u8(4);
    branch_root: DistributedMessage::BranchRoot(BranchRoot { username: "root".into() }) => p().s("root").frame_u8(5);
    child_depth: DistributedMessage::ChildDepth(ChildDepth { depth: 2 }) => p().u32(2).frame_u8(7);
    embedded_message:
      DistributedMessage::EmbeddedMessage(EmbeddedMessage {
        embedded: Embedded { distributed_code: 3, payload: search() },
        padded: false,
      })
      => p().u8(3).raw(&search()).frame_u8(93);
    embedded_message_padded:
      DistributedMessage::EmbeddedMessage(EmbeddedMessage {
        embedded: Embedded { distributed_code: 3, payload: search() },
        padded: true,
      })
      => p().raw(&[0, 0, 0]).u8(3).raw(&search()).frame_u8(93);
  }

  decodes! {
    search_request_rejects_an_identifier_other_than_49: DistributedMessage, p().u32(0).s("bob").u32(5).s("q").frame_u8(3)
      => Err(field("SearchRequest", "identifier", FieldError::BadSearchIdentifier { identifier: 0 }));
    branch_level_rejects_a_negative_level: DistributedMessage, p().i32(-1).frame_u8(4)
      => Err(field("BranchLevel", "level", FieldError::NegativeBranchLevel { level: -1 }));
    branch_root_rejects_a_non_printable_username: DistributedMessage, p().s("a\u{7}b").frame_u8(5)
      => Err(field("BranchRoot", "username", FieldError::InvalidUsername));
    embedded_message_with_a_short_zero_prefix_is_not_padded: DistributedMessage, p().raw(&[0, 0, 0]).frame_u8(93)
      => Ok(DistributedMessage::EmbeddedMessage(EmbeddedMessage {
        embedded: Embedded { distributed_code: 0, payload: vec![0, 0] },
        padded: false,
      }));
    unknown_placeholder_code: DistributedMessage, p().frame_u8(6) => Err(DecodeError::UnknownCode { code: 6 });
  }

  #[test]
  fn embedded_message_inner_frame_decodes_as_distributed() {
    let embedded = Embedded {
      distributed_code: 3,
      payload: search(),
    };
    assert_eq!(embedded.inner_frame(), p().raw(&search()).frame_u8(3));
    assert_eq!(
      DistributedMessage::decode(&embedded.inner_frame()),
      Ok(DistributedMessage::SearchRequest(SearchRequest {
        username: "bob".into(),
        token: 5,
        query: "q".into()
      }))
    );
  }
}
