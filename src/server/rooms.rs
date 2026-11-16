//! Chat room messages.
//!
//! Joining and leaving rooms, room membership, the room list and tickers.

use super::users::{UserData, UserStats};
use crate::codec::{Wire, wire_struct};
use crate::error::{DecodeError, FieldError};
use crate::log::KeyFields;
use crate::model::{RoomType, UserPresence};
use crate::wire::{Reader, Writer};

wire_struct! {
  /// Chat message sent to a room.
  ///
  /// Server code 13. Sent by the client.
  pub struct SayInChatRoomRequest {
    pub room: String,
    pub message: String,
  }
}

wire_struct! {
  /// Chat message in a joined room.
  ///
  /// Server code 13. Sent by the server for every message in a room the client has joined,
  /// including messages the client sent.
  pub struct SayInChatRoomResponse {
    pub room: String,
    pub username: String,
    pub message: String,
  }
}

wire_struct! {
  /// Request to join a room.
  ///
  /// Server code 14. Sent by the client. The server creates the room with `room_type` if it does
  /// not exist.
  pub struct JoinRoomRequest {
    pub room: String,
    /// Type of the room to create if it does not exist.
    pub room_type: RoomType,
  }
}

/// Members of a joined room.
///
/// Server code 14. Sent by the server when the client joins a room, with the owner and operators
/// of a private room.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct RoomData {
  pub name: String,
  /// Members of the room. For a private room, includes the owner and the operators.
  pub users: Vec<UserData>,
  /// Owner and operators, present if and only if the room is private.
  pub private: Option<PrivateRoomData>,
}

impl RoomData {
  /// Returns the number of members in the room.
  pub fn user_count(&self) -> usize {
    self.users.len()
  }

  /// Returns `true` if the room is private.
  ///
  /// The owner and operators of a private room are known.
  pub fn is_private(&self) -> bool {
    self.private.is_some()
  }
}

impl Wire for RoomData {
  const MIN_BYTES: usize = 24;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let name = r.string("name")?;
    let users = read_user_lists(r)?;
    let private = r.optional("private")?;
    Ok(Self {
      name,
      users,
      private,
    })
  }

  fn write(&self, w: &mut Writer) {
    w.string(&self.name);
    write_user_lists(w, &self.users);
    if let Some(private) = &self.private {
      private.write(w);
    }
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.room = Some(&self.name);
    fields.count = Some(self.users.len());
  }
}

wire_struct! {
  /// Owner and operators of a private room.
  ///
  /// Follows the member lists in [`RoomData`].
  pub struct PrivateRoomData {
    pub owner: String,
    pub operators: Vec<String>,
  }
}

/// a list that must have `n` elements, the length of the list it runs parallel to
fn read_parallel<T: Wire>(
  r: &mut Reader,
  n: usize,
  field: &'static str,
) -> Result<Vec<T>, DecodeError> {
  if r.count(T::MIN_BYTES, field)? != n {
    return Err(r.fail(field, FieldError::ListLengthMismatch));
  }
  (0..n).map(|_| T::read(r, field)).collect()
}

/// The five parallel per-user lists of `JoinRoom` and `GlobalUserList`, which must agree in
/// length
pub(super) fn read_user_lists(r: &mut Reader) -> Result<Vec<UserData>, DecodeError> {
  let usernames: Vec<String> = Wire::read(r, "users")?;
  let n = usernames.len();
  let statuses: Vec<UserPresence> = read_parallel(r, n, "statuses")?;
  let stats: Vec<UserStats> = read_parallel(r, n, "stats")?;
  let slots = read_parallel::<u32>(r, n, "slots_full")?
    .into_iter()
    .map(|slots_full| slots_full != 0);
  let countries: Vec<String> = read_parallel(r, n, "country_codes")?;
  Ok(
    usernames
      .into_iter()
      .zip(statuses)
      .zip(stats)
      .zip(slots)
      .zip(countries)
      .map(
        |((((username, status), stats), slots_full), country_code)| UserData {
          username,
          status,
          stats,
          slots_full,
          country_code,
        },
      )
      .collect(),
  )
}

pub(super) fn write_user_lists(w: &mut Writer, users: &[UserData]) {
  w.count(users.len());
  for user in users {
    w.string(&user.username);
  }
  w.count(users.len());
  for user in users {
    user.status.write(w);
  }
  w.count(users.len());
  for user in users {
    user.stats.write(w);
  }
  w.count(users.len());
  for user in users {
    w.u32(u32::from(user.slots_full));
  }
  w.count(users.len());
  for user in users {
    w.string(&user.country_code);
  }
}

wire_struct! {
  /// Request to leave a room.
  ///
  /// Server code 15. Sent by the client. The server echoes the message back once the client has
  /// left the room.
  pub struct LeaveRoom {
    pub room: String,
  }
}

/// Notification that a user joined a room.
///
/// Server code 16. Sent by the server for a room the client is in, including when the client
/// itself joins.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct UserJoinedRoom {
  pub room: String,
  pub user: UserData,
}

impl Wire for UserJoinedRoom {
  const MIN_BYTES: usize = 40;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      room: r.string("room")?,
      user: UserData {
        username: r.string("username")?,
        status: UserPresence::read(r, "status")?,
        stats: UserStats::read(r, "stats")?,
        slots_full: r.u32("slots_full")? != 0,
        country_code: r.string("country_code")?,
      },
    })
  }

  fn write(&self, w: &mut Writer) {
    w.string(&self.room);
    w.string(&self.user.username);
    self.user.status.write(w);
    self.user.stats.write(w);
    w.u32(u32::from(self.user.slots_full));
    w.string(&self.user.country_code);
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.room = Some(&self.room);
    fields.username = Some(&self.user.username);
  }
}

wire_struct! {
  /// Notification that a user left a room.
  ///
  /// Server code 17. Sent by the server for a room the client is in, when a user other than the
  /// client leaves.
  pub struct UserLeftRoom {
    pub room: String,
    pub username: String,
  }
}

wire_struct! {
  /// Notification that a room was created.
  ///
  /// Server code 62.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct RoomAdded {
    pub room: String,
  }
}

wire_struct! {
  /// Notification that a room was removed.
  ///
  /// Server code 63.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct RoomRemoved {
    pub room: String,
  }
}

/// One entry of [`RoomList`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct RoomInfo {
  pub name: String,
  pub user_count: u32,
}

/// the names and counts of one `RoomList` block, which must agree in length
fn read_room_block(
  r: &mut Reader,
  names_field: &'static str,
  counts_field: &'static str,
) -> Result<Vec<RoomInfo>, DecodeError> {
  let names: Vec<String> = Wire::read(r, names_field)?;
  let counts: Vec<u32> = read_parallel(r, names.len(), counts_field)?;
  Ok(
    names
      .into_iter()
      .zip(counts)
      .map(|(name, user_count)| RoomInfo { name, user_count })
      .collect(),
  )
}

fn write_room_block(w: &mut Writer, rooms: &[RoomInfo]) {
  w.count(rooms.len());
  for room in rooms {
    w.string(&room.name);
  }
  w.count(rooms.len());
  for room in rooms {
    w.u32(room.user_count);
  }
}

/// List of chat rooms with their user counts.
///
/// Server code 64. Sent by the server after login and in reply to a room list request. Holds the
/// public rooms, and the private rooms the client owns, is a member of and operates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct RoomList {
  pub public: Vec<RoomInfo>,
  /// Private rooms the client owns.
  pub owned: Vec<RoomInfo>,
  /// Private rooms the client is a member of but does not own.
  pub private: Vec<RoomInfo>,
  /// Private rooms the client operates.
  pub moderated_room_names: Vec<String>,
}

impl Wire for RoomList {
  const MIN_BYTES: usize = 28;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      public: read_room_block(r, "public", "public_user_counts")?,
      owned: read_room_block(r, "owned", "owned_user_counts")?,
      private: read_room_block(r, "private", "private_user_counts")?,
      moderated_room_names: Wire::read(r, "moderated_room_names")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    write_room_block(w, &self.public);
    write_room_block(w, &self.owned);
    write_room_block(w, &self.private);
    self.moderated_room_names.write(w);
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.count = Some(self.public.len());
  }
}

wire_struct! {
  /// Every ticker in a room.
  ///
  /// Server code 113. Sent by the server when the client joins the room.
  pub struct RoomTickers {
    pub room: String,
    pub tickers: Vec<RoomTicker>,
  }
}

wire_struct! {
  /// A user's ticker in a room.
  ///
  /// A ticker is the short status line shown next to the user in a room.
  pub struct RoomTicker {
    pub username: String,
    pub message: String,
  }
}

wire_struct! {
  /// Notification that a user set their ticker.
  ///
  /// Server code 114. Sent by the server for a room the client is in.
  pub struct RoomTickerAdd {
    pub room: String,
    pub ticker: RoomTicker,
  }
}

wire_struct! {
  /// Notification that a user cleared their ticker.
  ///
  /// Server code 115. Sent by the server for a room the client is in.
  pub struct RoomTickerRemove {
    pub room: String,
    pub username: String,
  }
}

wire_struct! {
  /// Request to set or clear the client's ticker in a room.
  ///
  /// Server code 116. Sent by the client.
  pub struct SetRoomTicker {
    pub room: String,
    /// Ticker text. An empty string clears the ticker.
    pub ticker: String,
  }
}

wire_struct! {
  /// Rejection of a request to join a room.
  ///
  /// Server code 1003. Sent by the server when a [`JoinRoomRequest`] would have created a room
  /// with the name of an existing private room.
  pub struct CannotJoinRoom {
    pub room: String,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::server::users::tests::{stats_bytes, user};
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{decodes, field, first, p, vectors};

  vectors! {
    say_in_chat_room_request:
      Q::SayInChatRoom(SayInChatRoomRequest { room: "indie".into(), message: "hi all".into() })
      => p().s("indie").s("hi all").frame(13);
    say_in_chat_room_response:
      R::SayInChatRoom(SayInChatRoomResponse { room: "indie".into(), username: "bob".into(), message: "hi".into() })
      => p().s("indie").s("bob").s("hi").frame(13);
    join_room_request:
      Q::JoinRoom(JoinRoomRequest { room: "indie".into(), room_type: RoomType::Private }) => p().s("indie").u32(1).frame(14);
    join_room_response:
      R::JoinRoom(RoomData {
        name: "indie".into(),
        users: vec![user("bob", UserPresence::Online, false, "NL"), user("eve", UserPresence::Away, true, "")],
        private: None,
      })
      => stats_bytes(stats_bytes(p().s("indie").u32(2).s("bob").s("eve").u32(2).u32(2).u32(1).u32(2)))
        .u32(2).u32(0).u32(1).u32(2).s("NL").s("").frame(14);
    join_room_response_private:
      R::JoinRoom(RoomData {
        name: "club".into(),
        users: vec![user("owner", UserPresence::Online, false, "DE")],
        private: Some(PrivateRoomData { owner: "owner".into(), operators: vec!["op".into()] }),
      })
      => stats_bytes(p().s("club").u32(1).s("owner").u32(1).u32(2).u32(1))
        .u32(1).u32(0).u32(1).s("DE").s("owner").u32(1).s("op").frame(14);
    leave_room_request: Q::LeaveRoom(LeaveRoom { room: "indie".into() }) => p().s("indie").frame(15);
    leave_room_response: R::LeaveRoom(LeaveRoom { room: "indie".into() }) => p().s("indie").frame(15);
    user_joined_room_response:
      R::UserJoinedRoom(UserJoinedRoom { room: "indie".into(), user: user("bob", UserPresence::Online, true, "FR") })
      => stats_bytes(p().s("indie").s("bob").u32(2)).u32(1).s("FR").frame(16);
    user_left_room_response:
      R::UserLeftRoom(UserLeftRoom { room: "indie".into(), username: "bob".into() }) => p().s("indie").s("bob").frame(17);
    room_added_response: R::RoomAdded(RoomAdded { room: "new".into() }) => p().s("new").frame(62);
    room_removed_response: R::RoomRemoved(RoomRemoved { room: "old".into() }) => p().s("old").frame(63);
    room_list_request: Q::RoomList => p().frame(64);
    room_list_response:
      R::RoomList(RoomList {
        public: vec![RoomInfo { name: "indie".into(), user_count: 40 }, RoomInfo { name: "jazz".into(), user_count: 7 }],
        owned: vec![RoomInfo { name: "mine".into(), user_count: 2 }],
        private: vec![],
        moderated_room_names: vec!["club".into()],
      })
      => p().u32(2).s("indie").s("jazz").u32(2).u32(40).u32(7).u32(1).s("mine").u32(1).u32(2)
        .u32(0).u32(0).u32(1).s("club").frame(64);
    room_tickers_response:
      R::RoomTickers(RoomTickers {
        room: "indie".into(),
        tickers: vec![RoomTicker { username: "bob".into(), message: "listening".into() }],
      })
      => p().s("indie").u32(1).s("bob").s("listening").frame(113);
    room_ticker_add_response:
      R::RoomTickerAdd(RoomTickerAdd { room: "indie".into(), ticker: RoomTicker { username: "bob".into(), message: "t".into() } })
      => p().s("indie").s("bob").s("t").frame(114);
    room_ticker_remove_response:
      R::RoomTickerRemove(RoomTickerRemove { room: "indie".into(), username: "bob".into() }) => p().s("indie").s("bob").frame(115);
    set_room_ticker_request:
      Q::SetRoomTicker(SetRoomTicker { room: "indie".into(), ticker: "".into() }) => p().s("indie").s("").frame(116);
    cannot_join_room_response: R::CannotJoinRoom(CannotJoinRoom { room: "club".into() }) => p().s("club").frame(1003);
  }

  decodes! {
    join_room_response_rejects_mismatched_lists: R, p().s("r").u32(2).s("a").s("b").u32(1).u32(2).frame(14)
      => Err(field("JoinRoom", "statuses", FieldError::ListLengthMismatch));
    leave_room_response_ignores_trailing_fields: R, p().s("room").raw(&[1, 2, 3]).frame(15)
      => Ok(R::LeaveRoom(LeaveRoom { room: "room".into() }));
    leave_room_response_rejects_a_string_past_the_payload: R, p().u32(10).raw(b"abc").frame(15)
      => Err(field("LeaveRoom", "room", FieldError::Truncated { needed: 10, remaining: 3 }));
    leave_room_response_rejects_a_cut_frame: R, first(p().s("room").frame(15), 10)
      => Err(DecodeError::IncompleteFrame { needed: 16, available: 10 });
    join_room_response_rejects_a_count_past_the_payload: R, p().s("r").u32(u32::MAX).frame(14)
      => Err(field("JoinRoom", "users", FieldError::CountTooLarge { count: u32::MAX, remaining: 0 }));
    room_list_response_rejects_mismatched_lists: R, p().u32(1).s("a").u32(2).u32(1).u32(1).frame(64)
      => Err(field("RoomList", "public_user_counts", FieldError::ListLengthMismatch));
    room_list_response_rejects_a_count_past_the_payload: R, p().u32(u32::MAX).frame(64)
      => Err(field("RoomList", "public", FieldError::CountTooLarge { count: u32::MAX, remaining: 0 }));
  }
}
