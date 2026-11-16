//! Server messages about the distributed search tree.
//!
//! They concern the client's place in the tree. The embedded message body is
//! [`crate::distributed::Embedded`].

use std::net::Ipv4Addr;

use crate::codec::{Wire, wire_struct};
use crate::error::DecodeError;
use crate::wire::{Reader, Writer};

wire_struct! {
  /// Whether the client lacks a distributed parent.
  ///
  /// Server code 71. Sent by the client. While the client lacks one, the server sends [`NetInfo`]
  /// candidates.
  pub struct HaveNoParents {
    pub have_no_parents: bool,
  }
}

/// Address of the client's distributed parent.
///
/// Server code 73. The payload is empty when the client has none.
///
/// Deprecated: never sent.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct ParentsIP {
  pub ip: Option<Ipv4Addr>,
}

impl Wire for ParentsIP {
  const MIN_BYTES: usize = 0;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    Ok(Self {
      ip: r.optional("ip")?,
    })
  }

  fn write(&self, w: &mut Writer) {
    if let Some(ip) = self.ip {
      w.ip4(ip);
    }
  }
}

wire_struct! {
  /// Upload speed a client needs to become a distributed parent.
  ///
  /// Server code 83. Sent by the server after login.
  pub struct ParentMinSpeed {
    pub speed: u32,
  }
}

wire_struct! {
  /// Ratio for the client's distributed child limit.
  ///
  /// Server code 84. Sent by the server after login. The ratio sets how many distributed children
  /// the client may take.
  pub struct ParentSpeedRatio {
    pub ratio: u32,
  }
}

wire_struct! {
  /// Distributed parent timeout, in seconds.
  ///
  /// Server code 86.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct ParentInactivityTimeout {
    pub seconds: u32,
  }
}

wire_struct! {
  /// Distributed search timeout, in seconds.
  ///
  /// Server code 87.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct SearchInactivityTimeout {
    pub seconds: u32,
  }
}

wire_struct! {
  /// Number of parent candidates to keep.
  ///
  /// Server code 88.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct MinimumParentsInCache {
    pub count: u32,
  }
}

wire_struct! {
  /// Distributed keep-alive interval, in seconds.
  ///
  /// Server code 90.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct DistributedAliveInterval {
    pub seconds: u32,
  }
}

wire_struct! {
  /// Whether the server may hand distributed children to the client.
  ///
  /// Server code 100. Sent by the client.
  pub struct AcceptChildren {
    pub accept: bool,
  }
}

wire_struct! {
  /// Up to ten distributed parent candidates.
  ///
  /// Server code 102. Sent by the server after `HaveNoParents(true)` until the client has a parent.
  pub struct NetInfo {
    pub parents: Vec<ParentCandidate>,
  }
}

wire_struct! {
  /// Distributed parent candidate.
  ///
  /// A user that [`NetInfo`] offers as a parent, with the address to connect to.
  pub struct ParentCandidate {
    pub username: String,
    pub ip: Ipv4Addr,
    pub port: u32,
  }
}

wire_struct! {
  /// Depth of the client in the distributed tree.
  ///
  /// Server code 126. Sent by the client. The depth is 0 when the client is a branch root.
  pub struct BranchLevel {
    pub level: u32,
  }
}

wire_struct! {
  /// Username at the root of the client's distributed branch.
  ///
  /// Server code 127. Sent by the client. The client's own username when the client is the root.
  pub struct BranchRoot {
    pub root: String,
  }
}

wire_struct! {
  /// Number of levels of distributed children below the client.
  ///
  /// Server code 129.
  ///
  /// Obsolete: never sent, the server works the depth out itself.
  pub struct ChildDepth {
    pub depth: u32,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::distributed::Embedded;
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{IP, ip, p, vectors};

  fn embedded_search() -> Vec<u8> {
    p().u32(49).s("bob").u32(5).s("query").0
  }

  vectors! {
    have_no_parents_request: Q::HaveNoParents(HaveNoParents { have_no_parents: true }) => p().u8(1).frame(71);
    parents_ip_request: Q::ParentsIP(ParentsIP { ip: Some(ip()) }) => p().ip(IP).frame(73);
    parents_ip_request_without_parent: Q::ParentsIP(ParentsIP { ip: None }) => p().frame(73);
    parent_min_speed_response: R::ParentMinSpeed(ParentMinSpeed { speed: 1 }) => p().u32(1).frame(83);
    parent_speed_ratio_response: R::ParentSpeedRatio(ParentSpeedRatio { ratio: 50 }) => p().u32(50).frame(84);
    parent_inactivity_timeout_response:
      R::ParentInactivityTimeout(ParentInactivityTimeout { seconds: 300 }) => p().u32(300).frame(86);
    search_inactivity_timeout_response:
      R::SearchInactivityTimeout(SearchInactivityTimeout { seconds: 0 }) => p().u32(0).frame(87);
    minimum_parents_in_cache_response: R::MinimumParentsInCache(MinimumParentsInCache { count: 10 }) => p().u32(10).frame(88);
    distributed_alive_interval_response:
      R::DistributedAliveInterval(DistributedAliveInterval { seconds: 0 }) => p().u32(0).frame(90);
    embedded_message_response:
      R::EmbeddedMessage(Embedded { distributed_code: 3, payload: embedded_search() })
      => p().u8(3).raw(&embedded_search()).frame(93);
    accept_children_request: Q::AcceptChildren(AcceptChildren { accept: false }) => p().u8(0).frame(100);
    net_info_response:
      R::NetInfo(NetInfo { parents: vec![ParentCandidate { username: "fast".into(), ip: ip(), port: 2234 }] })
      => p().u32(1).s("fast").ip(IP).u32(2234).frame(102);
    branch_level_request: Q::BranchLevel(BranchLevel { level: 2 }) => p().u32(2).frame(126);
    branch_root_request: Q::BranchRoot(BranchRoot { root: "root".into() }) => p().s("root").frame(127);
    child_depth_request: Q::ChildDepth(ChildDepth { depth: 1 }) => p().u32(1).frame(129);
    distributed_reset_response: R::DistributedReset => p().frame(130);
  }
}
