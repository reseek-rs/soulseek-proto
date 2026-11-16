//! Data model shared by messages.
//!
//! Enumerations carried by messages, and the parsing of a free-text search query.

use std::collections::HashSet;

use crate::error::FieldError;

/// Online state of a user.
///
/// Reported by `GetStatus` and the room messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum UserPresence {
  /// Not logged in.
  Offline,
  /// Logged in and marked away.
  Away,
  /// Logged in and not away.
  Online,
  /// Any other value, kept so that the message it arrives in still decodes.
  Unknown(#[cfg_attr(test, proptest(strategy = "3..=u32::MAX"))] u32),
}

impl UserPresence {
  pub(crate) fn from_u32(value: u32) -> Self {
    match value {
      0 => Self::Offline,
      1 => Self::Away,
      2 => Self::Online,
      other => Self::Unknown(other),
    }
  }

  pub(crate) fn to_u32(self) -> u32 {
    match self {
      Self::Offline => 0,
      Self::Away => 1,
      Self::Online => 2,
      Self::Unknown(other) => other,
    }
  }
}

/// Type of a room.
///
/// The type of room that a [`JoinRoomRequest`](crate::server::JoinRoomRequest) creates when the
/// room does not exist yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum RoomType {
  /// Anyone can join.
  Public = 0,
  /// Only members added by the owner or an operator can join.
  Private = 1,
}

impl RoomType {
  pub(crate) fn from_u32(value: u32) -> Result<Self, FieldError> {
    match value {
      0 => Ok(Self::Public),
      1 => Ok(Self::Private),
      _ => Err(FieldError::InvalidValue { value }),
    }
  }
}

/// Type of a peer connection.
///
/// A one-character connection type, carried by [`PeerInit`](crate::init::PeerInit) and
/// `ConnectToPeer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum PeerConnectionType {
  /// `"P"`: peer messages.
  Peer,
  /// `"F"`: raw file bytes.
  Transfer,
  /// `"D"`: the distributed search tree.
  Distributed,
}

impl PeerConnectionType {
  pub(crate) fn as_str(self) -> &'static str {
    match self {
      Self::Peer => "P",
      Self::Transfer => "F",
      Self::Distributed => "D",
    }
  }

  pub(crate) fn parse(value: &str) -> Result<Self, FieldError> {
    match value {
      "P" => Ok(Self::Peer),
      "F" => Ok(Self::Transfer),
      "D" => Ok(Self::Distributed),
      _ => Err(FieldError::UnknownConnectionType),
    }
  }
}

/// Type of a file attribute.
///
/// The value `3` (encoder) is obsolete and decodes as [`Unknown(3)`](Self::Unknown), as do values
/// from third-party clients, so that an attribute list re-encodes unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FileAttributeType {
  /// Bit rate, in kilobits per second.
  BitRate,
  /// Duration, in seconds, truncated.
  Duration,
  /// Variable bit rate flag, 0 or 1.
  VariableBitRate,
  /// Sample rate, in hertz.
  SampleRate,
  /// Bit depth, in bits.
  BitDepth,
  /// Any other type.
  Unknown(u32),
}

impl FileAttributeType {
  pub(crate) fn from_u32(value: u32) -> Self {
    match value {
      0 => Self::BitRate,
      1 => Self::Duration,
      2 => Self::VariableBitRate,
      4 => Self::SampleRate,
      5 => Self::BitDepth,
      other => Self::Unknown(other),
    }
  }

  pub(crate) fn to_u32(self) -> u32 {
    match self {
      Self::BitRate => 0,
      Self::Duration => 1,
      Self::VariableBitRate => 2,
      Self::SampleRate => 4,
      Self::BitDepth => 5,
      Self::Unknown(other) => other,
    }
  }
}

/// Free-text search query split into terms and exclusions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchQuery {
  /// Terms in order, including a lone `-`.
  pub terms: Vec<String>,
  /// Exclusions, without their leading `-` characters.
  ///
  /// Exact duplicates are removed. Matched case-insensitively.
  pub exclusions: Vec<String>,
}

impl SearchQuery {
  /// Parses a free-text query.
  ///
  /// Splits the text on runs of whitespace.
  ///
  /// A token longer than one character that starts with `-` is an exclusion. Any other token is
  /// a term.
  pub fn parse(text: &str) -> Self {
    let mut query = Self::default();
    let mut seen = HashSet::new();
    for token in text.split_whitespace() {
      if token.len() > 1 && token.starts_with('-') {
        let exclusion = token.trim_start_matches('-');
        if seen.insert(exclusion) {
          query.exclusions.push(exclusion.to_owned());
        }
      } else {
        query.terms.push(token.to_owned());
      }
    }
    query
  }

  /// Returns the terms joined with single spaces.
  ///
  /// The exclusions are left out.
  pub fn query(&self) -> String {
    self.terms.join(" ")
  }

  /// Returns the text a search request sends.
  ///
  /// The terms, followed by each exclusion with a leading `-`.
  pub fn search_text(&self) -> String {
    let mut text = self.query();
    for exclusion in &self.exclusions {
      text.push_str(" -");
      text.push_str(exclusion);
    }
    text
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn search_query_terms_and_exclusions() {
    let query = SearchQuery::parse(" a  b\t- -x\n--y -x ");
    assert_eq!(query.terms, ["a", "b", "-"]);
    assert_eq!(query.exclusions, ["x", "y"]);
    assert_eq!(query.query(), "a b -");
    assert_eq!(query.search_text(), "a b - -x -y");
  }

  #[test]
  fn search_query_dashes_only_is_an_empty_exclusion() {
    assert_eq!(SearchQuery::parse("--").exclusions, [""]);
  }

  #[cfg(feature = "serde")]
  #[test]
  fn enums_serialize_under_their_variant_names() {
    use serde_json::{from_value, json, to_value};

    assert_eq!(to_value(UserPresence::Away).unwrap(), json!("Away"));
    let unknown = json!({ "Unknown": 7 });
    assert_eq!(to_value(UserPresence::Unknown(7)).unwrap(), unknown);
    assert_eq!(
      from_value::<UserPresence>(unknown).unwrap(),
      UserPresence::Unknown(7)
    );
    let transfer = json!("Transfer");
    assert_eq!(to_value(PeerConnectionType::Transfer).unwrap(), transfer);
    assert_eq!(
      from_value::<PeerConnectionType>(transfer).unwrap(),
      PeerConnectionType::Transfer
    );
    assert!(from_value::<PeerConnectionType>(json!("F")).is_err());
  }
}
