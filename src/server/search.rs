//! Search messages.
//!
//! Searches sent through the server, and the searches the server asks the client to answer.

use crate::codec::wire_struct;
use crate::wire::RemoteFilename;

wire_struct! {
  /// Search of one room's members, by room id.
  ///
  /// Server code 25.
  ///
  /// Obsolete: replaced by [`RoomSearch`].
  pub struct FileSearchRoom {
    pub token: u32,
    pub room_id: u32,
    pub query: String,
  }
}

wire_struct! {
  /// Search of the whole network.
  ///
  /// Server code 26. Sent by the client. Peers send their results as
  /// [`SearchResponse`](crate::peer::SearchResponse) with the same token.
  pub struct FileSearchRequest {
    pub token: u32,
    pub query: String,
  }
}

wire_struct! {
  /// Search for the client to answer.
  ///
  /// Server code 26. Sent by the server for a room or user search that includes the client.
  pub struct FileSearchResponse {
    pub username: String,
    pub token: u32,
    pub query: String,
  }
}

wire_struct! {
  /// Search of one user's shares.
  ///
  /// Server code 42. Sent by the client. The server forwards it to that user.
  pub struct UserSearch {
    pub username: String,
    pub token: u32,
    pub query: String,
  }
}

wire_struct! {
  /// Search for other sources of exactly one file.
  ///
  /// Server code 65. Sent by the client.
  ///
  /// Obsolete: the search yields no results.
  pub struct ExactFileSearchRequest {
    pub token: u32,
    pub filename: RemoteFilename,
    pub path: RemoteFilename,
    pub file_size: u64,
    pub checksum: u32,
    pub unknown: u8,
  }
}

wire_struct! {
  /// Exact file search relayed by the server.
  ///
  /// Server code 65.
  ///
  /// Obsolete: the server no longer sends this message.
  pub struct ExactFileSearchResponse {
    pub username: String,
    pub token: u32,
    pub filename: RemoteFilename,
    pub path: RemoteFilename,
    pub file_size: u64,
    pub checksum: u32,
  }
}

wire_struct! {
  /// Network search from the wishlist.
  ///
  /// Server code 103. Sent by the client. The server rate-limits it to [`WishlistInterval`].
  pub struct WishlistSearch {
    pub token: u32,
    pub query: String,
  }
}

wire_struct! {
  /// Number of seconds to wait between wishlist searches.
  ///
  /// Server code 104. Sent by the server after login.
  pub struct WishlistInterval {
    pub seconds: u32,
  }
}

wire_struct! {
  /// Search of one room's members.
  ///
  /// Server code 120. Sent by the client. The server forwards it to each member.
  pub struct RoomSearch {
    pub room: String,
    pub token: u32,
    pub query: String,
  }
}

wire_struct! {
  /// Request for search terms related to a query.
  ///
  /// Server code 153. Sent by the client.
  ///
  /// Obsolete: the server returns an empty list.
  pub struct RelatedSearchRequest {
    pub query: String,
  }
}

wire_struct! {
  /// Search terms related to a query, with scores.
  ///
  /// Server code 153. Sent by the server.
  ///
  /// Obsolete: the list is always empty.
  pub struct RelatedSearchResponse {
    pub query: String,
    pub terms: Vec<RelatedTerm>,
  }
}

wire_struct! {
  /// Search term related to a query, with its score.
  ///
  /// Listed in [`RelatedSearchResponse`].
  pub struct RelatedTerm {
    pub term: String,
    pub score: u32,
  }
}

wire_struct! {
  /// Phrases to exclude from search results.
  ///
  /// Server code 160. Sent by the server after login. The server asks clients not to return search
  /// results for these phrases.
  pub struct ExcludedSearchPhrases {
    pub phrases: Vec<String>,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::server::{ServerRequest as Q, ServerResponse as R};
  use crate::testing::{p, vectors};

  vectors! {
    file_search_room_request:
      Q::FileSearchRoom(FileSearchRoom { token: 8, room_id: 3, query: "a b".into() }) => p().u32(8).u32(3).s("a b").frame(25);
    file_search_request:
      Q::FileSearch(FileSearchRequest { token: 8, query: "boards of canada".into() })
      => p().u32(8).s("boards of canada").frame(26);
    file_search_response:
      R::FileSearch(FileSearchResponse { username: "bob".into(), token: 3, query: "aphex".into() })
      => p().s("bob").u32(3).s("aphex").frame(26);
    user_search_request:
      Q::UserSearch(UserSearch { username: "bob".into(), token: 9, query: "flac".into() }) => p().s("bob").u32(9).s("flac").frame(42);
    exact_file_search_request:
      Q::ExactFileSearch(ExactFileSearchRequest {
        token: 3, filename: "a.mp3".into(), path: "music\\a".into(), file_size: 1 << 33, checksum: 77, unknown: 1,
      })
      => p().u32(3).s("a.mp3").s("music\\a").u64(1 << 33).u32(77).u8(1).frame(65);
    exact_file_search_response:
      R::ExactFileSearch(ExactFileSearchResponse {
        username: "bob".into(), token: 3, filename: "a.mp3".into(), path: "music".into(), file_size: 99, checksum: 1,
      })
      => p().s("bob").u32(3).s("a.mp3").s("music").u64(99).u32(1).frame(65);
    wishlist_search_request:
      Q::WishlistSearch(WishlistSearch { token: 10, query: "rare".into() }) => p().u32(10).s("rare").frame(103);
    wishlist_interval_response: R::WishlistInterval(WishlistInterval { seconds: 720 }) => p().u32(720).frame(104);
    room_search_request:
      Q::RoomSearch(RoomSearch { room: "indie".into(), token: 11, query: "ep".into() }) => p().s("indie").u32(11).s("ep").frame(120);
    related_search_request: Q::RelatedSearch(RelatedSearchRequest { query: "jazz".into() }) => p().s("jazz").frame(153);
    related_search_response:
      R::RelatedSearch(RelatedSearchResponse { query: "jazz".into(), terms: vec![RelatedTerm { term: "bebop".into(), score: 5 }] })
      => p().s("jazz").u32(1).s("bebop").u32(5).frame(153);
    excluded_search_phrases_response:
      R::ExcludedSearchPhrases(ExcludedSearchPhrases { phrases: vec!["bad phrase".into()] }) => p().u32(1).s("bad phrase").frame(160);
  }
}
