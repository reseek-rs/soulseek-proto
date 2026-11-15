//! Messages on the server connection.
//!
//! [`ServerRequest`] holds the messages the client sends, and [`ServerResponse`] the messages the
//! server sends, unprompted or in reply. A code whose layout differs between the two directions has
//! a `...Request` and a `...Response` struct; a code with one layout in both directions has one
//! struct used by both enums.

pub(crate) mod distributed;
pub(crate) mod interests;
pub(crate) mod messaging;
pub(crate) mod obsolete;
pub(crate) mod peers;
pub(crate) mod private_rooms;
pub(crate) mod privileges;
pub(crate) mod rooms;
pub(crate) mod search;
pub(crate) mod session;
pub(crate) mod users;

pub use distributed::*;
pub use interests::*;
pub use messaging::*;
pub use obsolete::*;
pub use peers::*;
pub use private_rooms::*;
pub use privileges::*;
pub use rooms::*;
pub use search::*;
pub use session::*;
pub use users::*;

use crate::codec::family;
use crate::codes::ServerCode;
use crate::distributed::Embedded;

family! {
  /// Every message the client sends on the server connection.
  pub enum ServerRequest {
    code: ServerCode,
    frame: Server,
    label: "server request",
    quiet: [],
    messages: {
      Login(LoginRequest),
      SetListenPort(SetListenPort),
      GetPeerAddress(GetPeerAddressRequest),
      WatchUser(WatchUserRequest),
      UnwatchUser(UnwatchUser),
      GetStatus(GetStatusRequest),
      /// Obsolete.
      IgnoreUser(IgnoreUser),
      /// Obsolete.
      UnignoreUser(UnignoreUser),
      SayInChatRoom(SayInChatRoomRequest),
      JoinRoom(JoinRoomRequest),
      LeaveRoom(LeaveRoom),
      ConnectToPeer(ConnectToPeerRequest),
      PrivateMessage(PrivateMessageRequest),
      AcknowledgePrivateMessage(AcknowledgePrivateMessage),
      /// Obsolete.
      FileSearchRoom(FileSearchRoom),
      FileSearch(FileSearchRequest),
      SetOnlineStatus(SetOnlineStatus),
      /// Write-liveness probe.
      ///
      /// The server never replies to it.
      Ping,
      /// Obsolete.
      SendConnectToken(SendConnectToken),
      /// Obsolete.
      SendSpeed(SendSpeed),
      SharedFoldersAndFiles(SharedFoldersAndFiles),
      GetUserStats(GetUserStatsRequest),
      /// Obsolete.
      QueuedDownloads(QueuedDownloadsRequest),
      UserSearch(UserSearch),
      /// Obsolete.
      SimilarRecommendations(SimilarRecommendationsRequest),
      LikedInterestAdd(LikedInterestAdd),
      LikedInterestRemove(LikedInterestRemove),
      /// Request for interests recommended for the client.
      GetPersonalRecommendations,
      /// Request for the likes the server has on record for the client.
      ///
      /// Obsolete.
      MyRecommendations,
      /// Request for interests recommended across the network.
      GetGlobalRecommendations,
      GetUserInterests(GetUserInterestsRequest),
      /// Obsolete.
      AdminCommand(AdminCommand),
      /// Obsolete.
      PlaceInLineRequest(PlaceInLineRequest),
      /// Obsolete.
      PlaceInLineResponse(PlaceInLineResponse),
      /// Request for the room list.
      RoomList,
      /// Obsolete.
      ExactFileSearch(ExactFileSearchRequest),
      /// Request for every online user.
      ///
      /// Obsolete.
      GlobalUserList,
      /// Obsolete.
      TunneledMessage(TunneledMessageRequest),
      HaveNoParents(HaveNoParents),
      /// Deprecated: never sent.
      ParentsIP(ParentsIP),
      /// Request for the privilege time the client has left.
      CheckPrivileges,
      AcceptChildren(AcceptChildren),
      WishlistSearch(WishlistSearch),
      /// Request for users whose interests are similar to the client's.
      GetPersonalSimilarUsers,
      GetInterestRecommendations(GetInterestRecommendationsRequest),
      GetInterestSimilarUsers(GetInterestSimilarUsersRequest),
      SetRoomTicker(SetRoomTicker),
      HatedInterestAdd(HatedInterestAdd),
      HatedInterestRemove(HatedInterestRemove),
      RoomSearch(RoomSearch),
      SendUploadSpeed(SendUploadSpeed),
      /// Deprecated.
      UserPrivileges(UserPrivilegesRequest),
      GivePrivileges(GivePrivileges),
      /// Deprecated.
      AcknowledgeNotifyPrivileges(AcknowledgeNotifyPrivileges),
      BranchLevel(BranchLevel),
      BranchRoot(BranchRoot),
      /// Obsolete: never sent.
      ChildDepth(ChildDepth),
      PrivateRoomAddUser(PrivateRoomAddUser),
      PrivateRoomRemoveUser(PrivateRoomRemoveUser),
      PrivateRoomDropMembership(PrivateRoomDropMembership),
      PrivateRoomDropOwnership(PrivateRoomDropOwnership),
      /// Obsolete.
      PrivateRoomUnknown(PrivateRoomUnknown),
      PrivateRoomToggle(PrivateRoomToggle),
      NewPassword(NewPassword),
      PrivateRoomAddOperator(PrivateRoomAddOperator),
      PrivateRoomRemoveOperator(PrivateRoomRemoveOperator),
      MessageUsers(MessageUsers),
      /// Request to start the public chat feed.
      AskPublicChat,
      /// Request to stop the public chat feed.
      StopPublicChat,
      /// Obsolete.
      RelatedSearch(RelatedSearchRequest),
      CannotConnect(CannotConnectRequest),
    }
  }
}

family! {
  /// Every message the server sends, unprompted or in reply.
  pub enum ServerResponse {
    code: ServerCode,
    frame: Server,
    label: "server response",
    // embedded messages arrive for every network search while the client is a branch root
    quiet: [EmbeddedMessage],
    messages: {
      Login(LoginResponse),
      GetPeerAddress(GetPeerAddressResponse),
      WatchUser(WatchUserResponse),
      GetStatus(UserStatus),
      /// Obsolete.
      IgnoreUser(IgnoreUser),
      /// Obsolete.
      UnignoreUser(UnignoreUser),
      SayInChatRoom(SayInChatRoomResponse),
      JoinRoom(RoomData),
      LeaveRoom(LeaveRoom),
      UserJoinedRoom(UserJoinedRoom),
      UserLeftRoom(UserLeftRoom),
      ConnectToPeer(ConnectToPeerResponse),
      PrivateMessage(PrivateMessageResponse),
      /// Search for the client to answer.
      FileSearch(FileSearchResponse),
      /// The server never sends this message.
      Ping,
      /// Obsolete.
      SendConnectToken(SendConnectToken),
      GetUserStats(UserStatistics),
      /// Obsolete.
      QueuedDownloads(QueuedDownloadsResponse),
      /// Notice of another login with the client's credentials.
      KickedFromServer,
      /// Obsolete.
      SimilarRecommendations(SimilarRecommendationsResponse),
      GetPersonalRecommendations(Recommendations),
      /// Obsolete.
      MyRecommendations(MyRecommendationsResponse),
      GetGlobalRecommendations(Recommendations),
      GetUserInterests(UserInterests),
      /// Obsolete.
      PlaceInLineRequest(PlaceInLineRequest),
      /// Obsolete.
      PlaceInLineResponse(PlaceInLineResponse),
      /// Obsolete.
      RoomAdded(RoomAdded),
      /// Obsolete.
      RoomRemoved(RoomRemoved),
      RoomList(RoomList),
      /// Obsolete.
      ExactFileSearch(ExactFileSearchResponse),
      GlobalAdminMessage(GlobalAdminMessage),
      /// Obsolete.
      GlobalUserList(GlobalUserListResponse),
      /// Obsolete.
      TunneledMessage(TunneledMessageResponse),
      PrivilegedUsers(PrivilegedUsers),
      ParentMinSpeed(ParentMinSpeed),
      ParentSpeedRatio(ParentSpeedRatio),
      /// Obsolete.
      ParentInactivityTimeout(ParentInactivityTimeout),
      /// Obsolete.
      SearchInactivityTimeout(SearchInactivityTimeout),
      /// Obsolete.
      MinimumParentsInCache(MinimumParentsInCache),
      /// Obsolete.
      DistributedAliveInterval(DistributedAliveInterval),
      /// Obsolete.
      AddPrivilegedUser(AddPrivilegedUser),
      CheckPrivileges(CheckPrivilegesResponse),
      /// Distributed message wrapped by the server.
      ///
      /// Sent to a branch root.
      EmbeddedMessage(Embedded),
      NetInfo(NetInfo),
      WishlistInterval(WishlistInterval),
      GetPersonalSimilarUsers(GetPersonalSimilarUsersResponse),
      GetInterestRecommendations(GetInterestRecommendationsResponse),
      GetInterestSimilarUsers(GetInterestSimilarUsersResponse),
      RoomTickers(RoomTickers),
      RoomTickerAdd(RoomTickerAdd),
      RoomTickerRemove(RoomTickerRemove),
      /// Deprecated.
      UserPrivileges(UserPrivilegesResponse),
      /// Acknowledged with [`AcknowledgeNotifyPrivileges`].
      ///
      /// Deprecated.
      NotifyPrivileges(NotifyPrivileges),
      /// Instruction to reset distributed connections.
      ///
      /// Tells the client to drop every distributed connection and look for a new parent.
      DistributedReset,
      PrivateRoomUsers(PrivateRoomUsers),
      PrivateRoomAddUser(PrivateRoomAddUser),
      PrivateRoomRemoveUser(PrivateRoomRemoveUser),
      /// Obsolete.
      PrivateRoomUnknown(PrivateRoomUnknown),
      PrivateRoomAdded(PrivateRoomAdded),
      PrivateRoomRemoved(PrivateRoomRemoved),
      PrivateRoomToggle(PrivateRoomToggle),
      NewPassword(NewPassword),
      PrivateRoomAddOperator(PrivateRoomAddOperator),
      PrivateRoomRemoveOperator(PrivateRoomRemoveOperator),
      PrivateRoomOperatorAdded(PrivateRoomOperatorAdded),
      PrivateRoomOperatorRemoved(PrivateRoomOperatorRemoved),
      PrivateRoomOwned(PrivateRoomOwned),
      PublicChat(PublicChat),
      /// Obsolete.
      RelatedSearch(RelatedSearchResponse),
      ExcludedSearchPhrases(ExcludedSearchPhrases),
      CannotConnect(CannotConnectResponse),
      CannotJoinRoom(CannotJoinRoom),
    }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::error::DecodeError;
  use crate::testing::{decodes, p};

  decodes! {
    unknown_code_response: ServerResponse, p().u32(1).frame(9999) => Err(DecodeError::UnknownCode { code: 9999 });
  }
}
