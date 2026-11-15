//! Messages about other users.
//!
//! Their address, status and statistics, and watching and ignoring them. [`UserStats`] and
//! [`UserData`], which the room messages also carry, are defined here.

use std::net::Ipv4Addr;

use crate::codec::{Wire, wire_struct};
use crate::error::DecodeError;
use crate::log::KeyFields;
use crate::model::UserPresence;
use crate::wire::{Reader, Writer};

/// User statistics tuple.
///
/// Carried by [`WatchUserResponse`], [`UserStatistics`] and the room messages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct UserStats {
  /// Speed of recent uploads in bytes per second.
  pub average_speed: u32,
  /// Number of completed uploads as counted by the server.
  ///
  /// A rolling 32-bit counter.
  pub upload_count: u32,
  /// A `u32` of unknown meaning that follows `upload_count`, kept as received.
  pub unknown: u32,
  pub file_count: u32,
  pub directory_count: u32,
}

impl Wire for UserStats {
  const MIN_BYTES: usize = 20;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      average_speed: r.u32("average_speed")?,
      upload_count: r.u32("upload_count")?,
      unknown: r.u32("unknown")?,
      file_count: r.u32("file_count")?,
      directory_count: r.u32("directory_count")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    w.u32(self.average_speed);
    w.u32(self.upload_count);
    w.u32(self.unknown);
    w.u32(self.file_count);
    w.u32(self.directory_count);
  }
}

/// Room member.
///
/// Described by [`RoomData`](super::RoomData), [`UserJoinedRoom`](super::UserJoinedRoom) and
/// [`GlobalUserListResponse`](super::GlobalUserListResponse). Those messages always carry
/// `slots_full` and `country_code`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct UserData {
  pub username: String,
  pub status: UserPresence,
  pub stats: UserStats,
  /// Whether every upload slot of the user is busy.
  pub slots_full: bool,
  /// Country code as ISO 3166-1 alpha-2, upper-case, possibly blank.
  pub country_code: String,
}

wire_struct! {
  /// Request for a user's address and listen port.
  ///
  /// Server code 3. Sent by the client.
  pub struct GetPeerAddressRequest {
    pub username: String,
  }
}

/// Address and listen port of a user.
///
/// Server code 3. Sent by the server. The address is `0.0.0.0` when the user is offline.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct GetPeerAddressResponse {
  pub username: String,
  pub ip: Ipv4Addr,
  pub port: u32,
  /// Absent in messages from older servers.
  pub obfuscation: Option<PeerAddressObfuscation>,
}

wire_struct! {
  /// Obfuscated port trailer of [`GetPeerAddressResponse`].
  ///
  /// The port is two bytes. Type 0 is none and type 1 is "rotated".
  pub struct PeerAddressObfuscation {
    pub obfuscation_type: u32,
    pub obfuscated_port: u16,
  }
}

impl Wire for GetPeerAddressResponse {
  const MIN_BYTES: usize = 12;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      username: r.string("username")?,
      ip: r.ip4("ip")?,
      port: r.u32("port")?,
      obfuscation: r.optional_fixed("obfuscation")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    w.string(&self.username);
    w.ip4(self.ip);
    w.u32(self.port);
    if let Some(obfuscation) = &self.obfuscation {
      obfuscation.write(w);
    }
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.username = Some(&self.username);
  }
}

wire_struct! {
  /// Request to watch a user's status.
  ///
  /// Server code 5. Sent by the client. The server then reports the user's status changes to the
  /// client.
  pub struct WatchUserRequest {
    pub username: String,
  }
}

/// Whether a watched user exists.
///
/// Server code 5. Sent by the server, with the user's status and statistics if the user exists.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct WatchUserResponse {
  pub username: String,
  /// Status and statistics of the user, `None` if the user does not exist.
  pub user: Option<WatchedUser>,
}

impl Wire for WatchUserResponse {
  const MIN_BYTES: usize = 5;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let username = r.string("username")?;
    let user = if r.bool("exists")? {
      Some(WatchedUser {
        status: UserPresence::read(r, "status")?,
        stats: UserStats::read(r, "stats")?,
        country_code: r.optional("country_code")?,
      })
    } else {
      None
    };
    Ok(Self { username, user })
  }

  fn write(&self, w: &mut Writer) {
    w.string(&self.username);
    w.bool(self.user.is_some());
    if let Some(user) = &self.user {
      user.status.write(w);
      user.stats.write(w);
      if let Some(country_code) = &user.country_code {
        w.string(country_code);
      }
    }
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.username = Some(&self.username);
  }
}

/// Status and statistics of a watched user.
///
/// Carried by [`WatchUserResponse`] for a user that exists.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct WatchedUser {
  pub status: UserPresence,
  pub stats: UserStats,
  /// Present only if the user is not offline, and possibly blank.
  pub country_code: Option<String>,
}

wire_struct! {
  /// Request to stop watching a user.
  ///
  /// Server code 6. Sent by the client. Stops the status reports started by a [`WatchUserRequest`].
  pub struct UnwatchUser {
    pub username: String,
  }
}

wire_struct! {
  /// Request for a user's presence and privilege state.
  ///
  /// Server code 7. Sent by the client.
  pub struct GetStatusRequest {
    pub username: String,
  }
}

wire_struct! {
  /// Presence and privilege state of a user.
  ///
  /// Server code 7. Sent by the server in reply to a [`GetStatusRequest`], and unprompted for
  /// watched users and room members.
  pub struct UserStatus {
    pub username: String,
    pub presence: UserPresence,
    pub is_privileged: bool,
  }
}

wire_struct! {
  /// Notification that the client ignores a user.
  ///
  /// Server code 11. Sent to the server.
  ///
  /// Obsolete: the message no longer has any effect.
  pub struct IgnoreUser {
    pub username: String,
  }
}

wire_struct! {
  /// Notification that the client stops ignoring a user.
  ///
  /// Server code 12. Sent to the server.
  ///
  /// Obsolete: the message no longer has any effect.
  pub struct UnignoreUser {
    pub username: String,
  }
}

wire_struct! {
  /// Request for a user's upload speed and share counts.
  ///
  /// Server code 36. Sent by the client.
  pub struct GetUserStatsRequest {
    pub username: String,
  }
}

wire_struct! {
  /// Upload speed and share counts of a user.
  ///
  /// Server code 36. Sent by the server in reply to a [`GetUserStatsRequest`], and unprompted for
  /// users who share a room with the client.
  pub struct UserStatistics {
    pub username: String,
    pub stats: UserStats,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{IP, P, decodes, ip, p, vectors};

  pub(crate) fn stats() -> UserStats {
    UserStats {
      average_speed: 51_200,
      upload_count: 12,
      unknown: 0,
      file_count: 3_400,
      directory_count: 210,
    }
  }

  /// the bytes of `stats()`
  pub(crate) fn stats_bytes(b: P) -> P {
    b.u32(51_200).u32(12).u32(0).u32(3_400).u32(210)
  }

  pub(crate) fn user(
    username: &str,
    status: UserPresence,
    slots_full: bool,
    country_code: &str,
  ) -> UserData {
    UserData {
      username: username.into(),
      status,
      stats: stats(),
      slots_full,
      country_code: country_code.into(),
    }
  }

  vectors! {
    get_peer_address_request: Q::GetPeerAddress(GetPeerAddressRequest { username: "bob".into() }) => p().s("bob").frame(3);
    get_peer_address_response:
      R::GetPeerAddress(GetPeerAddressResponse { username: "bob".into(), ip: ip(), port: 2234,
        obfuscation: Some(PeerAddressObfuscation { obfuscation_type: 1, obfuscated_port: 2235 }),
      })
      => p().s("bob").ip(IP).u32(2234).u32(1).u16(2235).frame(3);
    get_peer_address_response_without_trailer:
      R::GetPeerAddress(GetPeerAddressResponse { username: "bob".into(), ip: Ipv4Addr::UNSPECIFIED, port: 0, obfuscation: None })
      => p().s("bob").ip([0, 0, 0, 0]).u32(0).frame(3);
    watch_user_request: Q::WatchUser(WatchUserRequest { username: "bob".into() }) => p().s("bob").frame(5);
    watch_user_response:
      R::WatchUser(WatchUserResponse {
        username: "bob".into(),
        user: Some(WatchedUser { status: UserPresence::Online, stats: stats(), country_code: Some("NL".into()) }),
      })
      => stats_bytes(p().s("bob").u8(1).u32(2)).s("NL").frame(5);
    watch_user_response_blank_country:
      R::WatchUser(WatchUserResponse {
        username: "bob".into(),
        user: Some(WatchedUser { status: UserPresence::Away, stats: stats(), country_code: Some(String::new()) }),
      })
      => stats_bytes(p().s("bob").u8(1).u32(1)).s("").frame(5);
    watch_user_response_offline:
      R::WatchUser(WatchUserResponse {
        username: "bob".into(),
        user: Some(WatchedUser { status: UserPresence::Offline, stats: stats(), country_code: None }),
      })
      => stats_bytes(p().s("bob").u8(1).u32(0)).frame(5);
    watch_user_response_missing:
      R::WatchUser(WatchUserResponse { username: "nobody".into(), user: None }) => p().s("nobody").u8(0).frame(5);
    unwatch_user_request: Q::UnwatchUser(UnwatchUser { username: "bob".into() }) => p().s("bob").frame(6);
    get_status_request: Q::GetStatus(GetStatusRequest { username: "bob".into() }) => p().s("bob").frame(7);
    get_status_response:
      R::GetStatus(UserStatus { username: "bob".into(), presence: UserPresence::Away, is_privileged: true })
      => p().s("bob").u32(1).u8(1).frame(7);
    ignore_user_request: Q::IgnoreUser(IgnoreUser { username: "bob".into() }) => p().s("bob").frame(11);
    ignore_user_response: R::IgnoreUser(IgnoreUser { username: "bob".into() }) => p().s("bob").frame(11);
    unignore_user_request: Q::UnignoreUser(UnignoreUser { username: "bob".into() }) => p().s("bob").frame(12);
    unignore_user_response: R::UnignoreUser(UnignoreUser { username: "bob".into() }) => p().s("bob").frame(12);
    get_user_stats_request: Q::GetUserStats(GetUserStatsRequest { username: "bob".into() }) => p().s("bob").frame(36);
    get_user_stats_response:
      R::GetUserStats(UserStatistics { username: "bob".into(), stats: stats() }) => stats_bytes(p().s("bob")).frame(36);
  }

  decodes! {
    get_peer_address_response_ignores_a_partial_trailer: R, p().s("bob").ip(IP).u32(1).u32(1).frame(3)
      => Ok(R::GetPeerAddress(GetPeerAddressResponse { username: "bob".into(), ip: ip(), port: 1, obfuscation: None }));
    get_status_response_bool_accepts_any_non_zero: R, p().s("bob").u32(2).u8(7).frame(7)
      => Ok(R::GetStatus(UserStatus { username: "bob".into(), presence: UserPresence::Online, is_privileged: true }));
    get_status_response_keeps_an_unknown_presence: R, p().s("bob").u32(3).u8(0).frame(7)
      => Ok(R::GetStatus(UserStatus { username: "bob".into(), presence: UserPresence::Unknown(3), is_privileged: false }));
  }
}
