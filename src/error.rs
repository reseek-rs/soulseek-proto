/// Reason a frame or message was rejected.
///
/// Remote strings are never carried in this type, so an error can be logged as is.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeError {
  /// Length prefix smaller than the code width.
  ///
  /// The stream cannot be resynchronized, and the connection should be closed.
  #[error("frame length {len} is below the code width {code_width}")]
  FrameTooShort { len: u32, code_width: usize },
  /// Length prefix over the limit for the connection kind, or for the code of a peer message.
  ///
  /// Over the limit for the connection kind, the connection should be closed. A peer message
  /// decoder also returns this for a frame longer than
  /// [`Limits::peer_message_bytes`](crate::Limits::peer_message_bytes) allows its code; that
  /// message is dropped.
  #[error("frame length {len} exceeds the limit of {limit} bytes")]
  FrameTooLarge { len: u32, limit: u32 },
  /// Input that ends before the frame does.
  ///
  /// The bytes given end before the end of the frame their length prefix announces. Decoders
  /// take whole frames, so the caller passed a partial frame. The message is dropped.
  #[error("the frame needs {needed} bytes but only {available} were given")]
  IncompleteFrame { needed: usize, available: usize },
  /// Bytes after the single frame or raw value a decoder takes.
  ///
  /// The message is dropped.
  #[error("{extra} bytes follow the frame")]
  TrailingBytes { extra: usize },
  /// Message code with no known layout.
  ///
  /// The message is dropped and the connection stays open, except for an init frame, where the
  /// socket should be closed.
  #[error("unknown message code {code}")]
  UnknownCode { code: u32 },
  /// Field of a known message that could not be read.
  ///
  /// The message is dropped.
  #[error("{message} {field}: {error}")]
  Field {
    message: &'static str,
    field: &'static str,
    error: FieldError,
  },
}

/// Problem with one field of a message.
///
/// The display text follows the message and field name, as in
/// "Login greeting: needs 12 bytes, 3 remain".
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FieldError {
  /// The payload ends before the field does.
  #[error("needs {needed} bytes, {remaining} remain")]
  Truncated { needed: usize, remaining: usize },
  /// String longer than its limit, in bytes.
  ///
  /// `len` is the length on the wire, or the length as UTF-8 for a string read as ISO-8859-1.
  #[error("length {len} exceeds the limit of {limit} bytes")]
  StringTooLong { len: u32, limit: usize },
  /// List count that the remaining bytes cannot hold.
  #[error("{count} elements cannot fit in the {remaining} remaining bytes")]
  CountTooLarge { count: u32, remaining: usize },
  /// Count or length over one of the [`Limits`](crate::Limits) or `LIMIT_*` constants.
  #[error("{value} exceeds the limit of {limit}")]
  LimitExceeded { value: u64, limit: u64 },
  /// List whose count differs from the list it runs parallel to.
  #[error("length differs from the earlier list")]
  ListLengthMismatch,
  /// Value outside the set the field allows.
  #[error("invalid value {value}")]
  InvalidValue { value: u32 },
  /// Connection type other than `P`, `F` or `D`.
  #[error("unknown connection type")]
  UnknownConnectionType,
  /// Username a peer may not state: empty, too long, with control characters, or `server`.
  #[error("not a valid username")]
  InvalidUsername,
  /// Negative branch level in a distributed message.
  #[error("negative level {level}")]
  NegativeBranchLevel { level: i32 },
  /// Distributed search whose first field is not
  /// [`SEARCH_IDENTIFIER`](crate::SEARCH_IDENTIFIER).
  #[error("identifier {identifier} is not 49")]
  BadSearchIdentifier { identifier: u32 },
  /// Compressed payload that is not a valid zlib stream.
  #[error("not a zlib stream")]
  InvalidZlib,
  /// Compressed payload that ends before the zlib stream does.
  #[error("the zlib stream ends before its end marker")]
  TruncatedZlib,
  /// Compressed payload that inflates past its limit, in bytes.
  #[error("inflates past the limit of {limit} bytes")]
  InflatedTooLarge { limit: usize },
}
