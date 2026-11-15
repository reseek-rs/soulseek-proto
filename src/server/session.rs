//! Session messages.
//!
//! Login, the listen port, online status, ping, share counts, kicks from the server, upload speed
//! and password changes.

use std::net::Ipv4Addr;

use md5::{Digest, Md5};

use crate::codec::{Wire, wire_struct};
use crate::error::DecodeError;
use crate::model::UserPresence;
use crate::wire::{Reader, Secret, Writer};

/// Major version sent in [`LoginRequest`].
///
/// The version registry keeps 177 for experimental clients. 170 belongs to another client project
/// and must not be reused.
pub const PROTOCOL_MAJOR_VERSION: u32 = 177;
/// Minor version sent in [`LoginRequest`].
///
/// The server only requires it to be non-zero.
pub const PROTOCOL_MINOR_VERSION: u32 = 760;

pub(crate) fn login_hash(username: &str, password: &str) -> String {
  let mut md5 = Md5::new();
  md5.update(username.as_bytes());
  md5.update(password.as_bytes());
  md5.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

wire_struct! {
  /// Login request.
  ///
  /// Server code 1. Sent by the client as the first message on the server connection, with the
  /// client's username, password and client version.
  pub struct LoginRequest {
    pub username: String,
    pub password: Secret,
    pub major_version: u32,
    /// Lowercase hexadecimal MD5 digest of the username followed by the password.
    pub hash: Secret,
    pub minor_version: u32,
  }
}

impl LoginRequest {
  /// Returns a login request with the hash computed from the credentials.
  pub fn new(username: String, password: String, major_version: u32, minor_version: u32) -> Self {
    let hash = Secret::new(login_hash(&username, &password));
    Self {
      username,
      password: Secret::new(password),
      major_version,
      hash,
      minor_version,
    }
  }
}

/// Acceptance or rejection of a login.
///
/// Server code 1. Sent by the server. An acceptance carries a greeting and the client's public
/// address, a rejection carries a reason.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum LoginResponse {
  /// The login was accepted.
  Success {
    /// Multi-line greeting from the server.
    greeting: String,
    /// Address of the client as the server sees it.
    ip: Ipv4Addr,
    /// Lowercase hexadecimal MD5 digest of the password the client sent, for information only.
    hash: Secret,
    /// Whether the client's account has ever bought privileges.
    is_supporter: bool,
  },
  /// The login was rejected.
  Failure {
    /// Reason for the rejection, such as `INVALIDUSERNAME`, `INVALIDPASS` or `SVRFULL`.
    reason: String,
    detail: Option<String>,
  },
}

impl Wire for LoginResponse {
  const MIN_BYTES: usize = 5;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(if r.bool_exact("success")? {
      Self::Success {
        greeting: r.description("greeting")?,
        ip: r.ip4("ip")?,
        hash: Secret::read(r, "hash")?,
        is_supporter: r.bool_exact("is_supporter")?,
      }
    } else {
      Self::Failure {
        reason: r.string("reason")?,
        detail: r.optional("detail")?,
      }
    })
  }

  fn write(&self, w: &mut Writer) {
    match self {
      Self::Success {
        greeting,
        ip,
        hash,
        is_supporter,
      } => {
        w.bool(true);
        w.string(greeting);
        w.ip4(*ip);
        hash.write(w);
        w.bool(*is_supporter);
      }
      Self::Failure { reason, detail } => {
        w.bool(false);
        w.string(reason);
        if let Some(detail) = detail {
          w.string(detail);
        }
      }
    }
  }
}

/// Port on which peers can reach the client.
///
/// Server code 2. Sent by the client with the login and whenever the port changes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct SetListenPort {
  pub port: u32,
  /// Omitted from the frame when `None`.
  pub obfuscation: Option<Obfuscation>,
}

wire_struct! {
  /// Obfuscated port trailer.
  ///
  /// Ends [`SetListenPort`] and [`ConnectToPeerResponse`](super::ConnectToPeerResponse). Type 0 is
  /// none and type 1 is "rotated".
  pub struct Obfuscation {
    pub obfuscation_type: u32,
    pub obfuscated_port: u32,
  }
}

impl Wire for SetListenPort {
  const MIN_BYTES: usize = 4;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      port: r.u32("port")?,
      obfuscation: r.optional_fixed("obfuscation")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    w.u32(self.port);
    if let Some(obfuscation) = &self.obfuscation {
      obfuscation.write(w);
    }
  }
}

wire_struct! {
  /// Presence of the client, away or online.
  ///
  /// Server code 28. Sent by the client whenever the presence changes.
  pub struct SetOnlineStatus {
    pub status: UserPresence,
  }
}

wire_struct! {
  /// Number of folders and files the client shares.
  ///
  /// Server code 35. Sent by the client after login and after each share scan.
  pub struct SharedFoldersAndFiles {
    pub directory_count: u32,
    pub file_count: u32,
  }
}

wire_struct! {
  /// Average speed of a finished upload.
  ///
  /// Server code 121. Sent by the client when it finishes an upload. The server includes the speed
  /// in the client's statistics.
  pub struct SendUploadSpeed {
    /// Speed in bytes per second.
    pub speed: u32,
  }
}

wire_struct! {
  /// New password for the client's account.
  ///
  /// Server code 142. Sent by the client. The server echoes the message back once the password is
  /// changed.
  pub struct NewPassword {
    pub new_password: Secret,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::error::FieldError;
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{IP, decodes, field, ip, p, vectors};

  const MD5_USERNAME_PASSWORD: &str = "d51c9a7e9353746a6020f9602d452929";
  const MD5_PASSWORD: &str = "5f4dcc3b5aa765d61d8327deb882cf99";

  vectors! {
    login_request:
      Q::Login(LoginRequest::new("username".into(), "password".into(), 177, 760))
      => p().s("username").s("password").u32(177).s(MD5_USERNAME_PASSWORD).u32(760).frame(1);
    login_response_success:
      R::Login(LoginResponse::Success {
        greeting: "welcome\nto soulseek".into(), ip: ip(), hash: Secret::new(MD5_PASSWORD), is_supporter: true,
      })
      => p().u8(1).s("welcome\nto soulseek").ip(IP).s(MD5_PASSWORD).u8(1).frame(1);
    login_response_failure:
      R::Login(LoginResponse::Failure { reason: "INVALIDPASS".into(), detail: None }) => p().u8(0).s("INVALIDPASS").frame(1);
    login_response_failure_with_detail:
      R::Login(LoginResponse::Failure { reason: "INVALIDUSERNAME".into(), detail: Some("Nick too long.".into()) })
      => p().u8(0).s("INVALIDUSERNAME").s("Nick too long.").frame(1);
    set_listen_port_request: Q::SetListenPort(SetListenPort { port: 50_000, obfuscation: None }) => p().u32(50_000).frame(2);
    set_listen_port_request_obfuscated:
      Q::SetListenPort(SetListenPort { port: 50_000, obfuscation: Some(Obfuscation { obfuscation_type: 1, obfuscated_port: 50_001 }) })
      => p().u32(50_000).u32(1).u32(50_001).frame(2);
    set_online_status_request:
      Q::SetOnlineStatus(SetOnlineStatus { status: UserPresence::Away }) => p().u32(1).frame(28);
    ping_request: Q::Ping => p().frame(32);
    ping_response: R::Ping => p().frame(32);
    shared_folders_and_files_request:
      Q::SharedFoldersAndFiles(SharedFoldersAndFiles { directory_count: 12, file_count: 345 }) => p().u32(12).u32(345).frame(35);
    kicked_from_server_response: R::KickedFromServer => p().frame(41);
    send_upload_speed_request: Q::SendUploadSpeed(SendUploadSpeed { speed: 123_456 }) => p().u32(123_456).frame(121);
    new_password_request: Q::NewPassword(NewPassword { new_password: Secret::new("n3w") }) => p().s("n3w").frame(142);
    new_password_response: R::NewPassword(NewPassword { new_password: Secret::new("n3w") }) => p().s("n3w").frame(142);
  }

  decodes! {
    login_response_success_must_be_exactly_one: R, p().u8(2).s("INVALIDPASS").frame(1)
      => Ok(R::Login(LoginResponse::Failure { reason: "INVALIDPASS".into(), detail: None }));
    login_response_success_without_supporter_flag_is_truncated: R,
      p().u8(1).s("welcome").ip(IP).s(MD5_PASSWORD).frame(1)
      => Err(field("Login", "is_supporter", FieldError::Truncated { needed: 1, remaining: 0 }));
    login_response_rejects_a_greeting_past_the_payload: R, p().u8(1).u32(100).raw(b"abc").frame(1)
      => Err(field("Login", "greeting", FieldError::Truncated { needed: 100, remaining: 3 }));
    ping_response_rejects_bytes_after_the_frame: R, [p().frame(32), vec![0]].concat()
      => Err(DecodeError::TrailingBytes { extra: 1 });
    set_listen_port_response_is_not_a_server_message: R, p().u32(1).frame(2)
      => Err(DecodeError::UnknownCode { code: 2 });
  }

  #[test]
  fn login_request_debug_never_shows_credentials() {
    let login = LoginRequest::new("alice".into(), "hunter2".into(), 1, 2);
    let debug = format!("{login:?}");
    assert!(!debug.contains("hunter2"));
    assert!(!debug.contains(&login_hash("alice", "hunter2")));
    assert!(debug.contains("alice"));
  }
}
