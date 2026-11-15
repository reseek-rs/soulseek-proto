//! Privilege messages.
//!
//! Which users have privileges, how long the client's privileges last, and giving them away.

use crate::codec::wire_struct;

wire_struct! {
  /// Every privileged user.
  ///
  /// Server code 69. Sent by the server after login.
  pub struct PrivilegedUsers {
    pub usernames: Vec<String>,
  }
}

wire_struct! {
  /// Notification that a user became privileged.
  ///
  /// Server code 91.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct AddPrivilegedUser {
    pub username: String,
  }
}

wire_struct! {
  /// Number of seconds of privileges the client has left.
  ///
  /// Server code 92. Sent by the server in reply to
  /// [`CheckPrivileges`](crate::ServerRequest::CheckPrivileges).
  pub struct CheckPrivilegesResponse {
    pub seconds_remaining: u32,
  }
}

wire_struct! {
  /// Request to know whether a user is privileged.
  ///
  /// Server code 122. Sent by the client.
  ///
  /// Deprecated: `GetStatus` carries the same flag.
  pub struct UserPrivilegesRequest {
    pub username: String,
  }
}

wire_struct! {
  /// Whether a user is privileged.
  ///
  /// Server code 122. Sent by the server.
  ///
  /// Deprecated: `GetStatus` carries the same flag.
  pub struct UserPrivilegesResponse {
    pub username: String,
    pub privileged: bool,
  }
}

wire_struct! {
  /// Request to give privilege days to another user.
  ///
  /// Server code 123. Sent by the client. The days come out of the client's own privileges.
  pub struct GivePrivileges {
    pub username: String,
    pub days: u32,
  }
}

wire_struct! {
  /// Notification that a user gave the client privileges.
  ///
  /// Server code 124. Sent by the server.
  ///
  /// Acknowledged with [`AcknowledgeNotifyPrivileges`].
  ///
  /// Deprecated: only Soulseek NS sends it.
  pub struct NotifyPrivileges {
    pub id: u32,
    pub username: String,
  }
}

wire_struct! {
  /// Acknowledgement of a [`NotifyPrivileges`].
  ///
  /// Server code 125. Sent by the client.
  ///
  /// Deprecated together with [`NotifyPrivileges`].
  pub struct AcknowledgeNotifyPrivileges {
    pub id: u32,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{p, vectors};

  vectors! {
    privileged_users_response:
      R::PrivilegedUsers(PrivilegedUsers { usernames: vec!["bob".into(), "eve".into()] }) => p().u32(2).s("bob").s("eve").frame(69);
    add_privileged_user_response: R::AddPrivilegedUser(AddPrivilegedUser { username: "bob".into() }) => p().s("bob").frame(91);
    check_privileges_request: Q::CheckPrivileges => p().frame(92);
    check_privileges_response:
      R::CheckPrivileges(CheckPrivilegesResponse { seconds_remaining: 86_400 }) => p().u32(86_400).frame(92);
    user_privileges_request: Q::UserPrivileges(UserPrivilegesRequest { username: "bob".into() }) => p().s("bob").frame(122);
    user_privileges_response:
      R::UserPrivileges(UserPrivilegesResponse { username: "bob".into(), privileged: true }) => p().s("bob").u8(1).frame(122);
    give_privileges_request:
      Q::GivePrivileges(GivePrivileges { username: "bob".into(), days: 30 }) => p().s("bob").u32(30).frame(123);
    notify_privileges_response:
      R::NotifyPrivileges(NotifyPrivileges { id: 4, username: "bob".into() }) => p().u32(4).s("bob").frame(124);
    acknowledge_notify_privileges_request:
      Q::AcknowledgeNotifyPrivileges(AcknowledgeNotifyPrivileges { id: 6 }) => p().u32(6).frame(125);
  }
}
