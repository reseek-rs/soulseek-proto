//! Private room membership and operators.

use crate::codec::wire_struct;

wire_struct! {
  /// Members of a private room the client is in.
  ///
  /// Server code 133. Sent by the server. Excludes the owner.
  pub struct PrivateRoomUsers {
    pub room: String,
    pub usernames: Vec<String>,
  }
}

wire_struct! {
  /// Request to add a member to a private room.
  ///
  /// Server code 134. Sent by the client for a room it owns or operates. The server sends the same
  /// message to every member once the member is added.
  pub struct PrivateRoomAddUser {
    pub room: String,
    pub username: String,
  }
}

wire_struct! {
  /// Request to remove a member from a private room.
  ///
  /// Server code 135. Sent by the client for a room it owns or operates. The server sends the same
  /// message to every member once the member is removed.
  pub struct PrivateRoomRemoveUser {
    pub room: String,
    pub username: String,
  }
}

wire_struct! {
  /// Request to give up membership of a private room.
  ///
  /// Server code 136. Sent by the client.
  pub struct PrivateRoomDropMembership {
    pub room: String,
  }
}

wire_struct! {
  /// Request to give up ownership of a private room.
  ///
  /// Server code 137. Sent by the client.
  pub struct PrivateRoomDropOwnership {
    pub room: String,
  }
}

wire_struct! {
  /// Message that names a private room.
  ///
  /// Server code 138. Its purpose is unknown.
  ///
  /// Obsolete: no longer used.
  pub struct PrivateRoomUnknown {
    pub room: String,
  }
}

wire_struct! {
  /// Notification that the client joined a private room.
  ///
  /// Server code 139. Sent by the server when the client is made a member of the room.
  pub struct PrivateRoomAdded {
    pub room: String,
  }
}

wire_struct! {
  /// Notification that the client left a private room.
  ///
  /// Server code 140. Sent by the server when the client is no longer a member of the room.
  pub struct PrivateRoomRemoved {
    pub room: String,
  }
}

wire_struct! {
  /// Whether the client accepts private room invitations.
  ///
  /// Server code 141. Sent by the client after login. The server echoes the message back.
  pub struct PrivateRoomToggle {
    pub accept_invitations: bool,
  }
}

wire_struct! {
  /// Request to make a member an operator of a private room.
  ///
  /// Server code 143. Sent by the client for a room it owns. The server sends the same message to
  /// every member once the operator is added.
  pub struct PrivateRoomAddOperator {
    pub room: String,
    pub username: String,
  }
}

wire_struct! {
  /// Request to remove an operator of a private room.
  ///
  /// Server code 144. Sent by the client for a room it owns. The server sends the same message to
  /// every member once the operator is removed.
  pub struct PrivateRoomRemoveOperator {
    pub room: String,
    pub username: String,
  }
}

wire_struct! {
  /// Notification that the client became a room operator.
  ///
  /// Server code 145. Sent by the server when the client is made an operator of a private room.
  pub struct PrivateRoomOperatorAdded {
    pub room: String,
  }
}

wire_struct! {
  /// Notification that the client lost room operator status.
  ///
  /// Server code 146. Sent by the server when the client is no longer an operator of a private
  /// room.
  pub struct PrivateRoomOperatorRemoved {
    pub room: String,
  }
}

wire_struct! {
  /// Operators of a private room the client is in.
  ///
  /// Server code 148. Sent by the server.
  pub struct PrivateRoomOwned {
    pub room: String,
    pub operators: Vec<String>,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{p, vectors};

  vectors! {
    private_room_users_response:
      R::PrivateRoomUsers(PrivateRoomUsers { room: "club".into(), usernames: vec!["bob".into()] })
      => p().s("club").u32(1).s("bob").frame(133);
    private_room_add_user_request:
      Q::PrivateRoomAddUser(PrivateRoomAddUser { room: "club".into(), username: "bob".into() }) => p().s("club").s("bob").frame(134);
    private_room_add_user_response:
      R::PrivateRoomAddUser(PrivateRoomAddUser { room: "club".into(), username: "bob".into() }) => p().s("club").s("bob").frame(134);
    private_room_remove_user_request:
      Q::PrivateRoomRemoveUser(PrivateRoomRemoveUser { room: "club".into(), username: "bob".into() })
      => p().s("club").s("bob").frame(135);
    private_room_remove_user_response:
      R::PrivateRoomRemoveUser(PrivateRoomRemoveUser { room: "club".into(), username: "bob".into() })
      => p().s("club").s("bob").frame(135);
    private_room_drop_membership_request:
      Q::PrivateRoomDropMembership(PrivateRoomDropMembership { room: "club".into() }) => p().s("club").frame(136);
    private_room_drop_ownership_request:
      Q::PrivateRoomDropOwnership(PrivateRoomDropOwnership { room: "club".into() }) => p().s("club").frame(137);
    private_room_unknown_request: Q::PrivateRoomUnknown(PrivateRoomUnknown { room: "club".into() }) => p().s("club").frame(138);
    private_room_unknown_response: R::PrivateRoomUnknown(PrivateRoomUnknown { room: "club".into() }) => p().s("club").frame(138);
    private_room_added_response: R::PrivateRoomAdded(PrivateRoomAdded { room: "club".into() }) => p().s("club").frame(139);
    private_room_removed_response: R::PrivateRoomRemoved(PrivateRoomRemoved { room: "club".into() }) => p().s("club").frame(140);
    private_room_toggle_request:
      Q::PrivateRoomToggle(PrivateRoomToggle { accept_invitations: true }) => p().u8(1).frame(141);
    private_room_toggle_response:
      R::PrivateRoomToggle(PrivateRoomToggle { accept_invitations: false }) => p().u8(0).frame(141);
    private_room_add_operator_request:
      Q::PrivateRoomAddOperator(PrivateRoomAddOperator { room: "club".into(), username: "bob".into() })
      => p().s("club").s("bob").frame(143);
    private_room_add_operator_response:
      R::PrivateRoomAddOperator(PrivateRoomAddOperator { room: "club".into(), username: "bob".into() })
      => p().s("club").s("bob").frame(143);
    private_room_remove_operator_request:
      Q::PrivateRoomRemoveOperator(PrivateRoomRemoveOperator { room: "club".into(), username: "bob".into() })
      => p().s("club").s("bob").frame(144);
    private_room_remove_operator_response:
      R::PrivateRoomRemoveOperator(PrivateRoomRemoveOperator { room: "club".into(), username: "bob".into() })
      => p().s("club").s("bob").frame(144);
    private_room_operator_added_response:
      R::PrivateRoomOperatorAdded(PrivateRoomOperatorAdded { room: "club".into() }) => p().s("club").frame(145);
    private_room_operator_removed_response:
      R::PrivateRoomOperatorRemoved(PrivateRoomOperatorRemoved { room: "club".into() }) => p().s("club").frame(146);
    private_room_owned_response:
      R::PrivateRoomOwned(PrivateRoomOwned { room: "club".into(), operators: vec!["op".into()] })
      => p().s("club").u32(1).s("op").frame(148);
  }
}
