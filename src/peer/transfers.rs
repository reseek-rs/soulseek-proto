//! Transfer negotiation.
//!
//! Requests, queueing, queue places and refusals.

use crate::codec::{Wire, wire_struct};
use crate::error::{DecodeError, FieldError};
use crate::log::KeyFields;
use crate::wire::{Reader, RemoteFilename, Writer};

/// Download request or upload announcement.
///
/// Peer code 40. Either a request to download a file, or an announcement by the uploader that it
/// is about to send one.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum TransferRequest {
  /// The sender wants to receive the file.
  ///
  /// Direction 0. A size some clients write after the filename is ignored.
  Download {
    /// Token chosen by the sender.
    token: u32,
    filename: RemoteFilename,
  },
  /// The sender is about to send the file.
  ///
  /// Direction 1.
  Upload {
    /// Token chosen by the sender.
    token: u32,
    filename: RemoteFilename,
    /// Size of the file, in bytes.
    ///
    /// Missing from very old clients.
    file_size: Option<u64>,
  },
}

impl TransferRequest {
  /// Returns the token chosen by the sender.
  pub fn token(&self) -> u32 {
    match self {
      Self::Download { token, .. } | Self::Upload { token, .. } => *token,
    }
  }

  /// Returns the file the request is about.
  pub fn filename(&self) -> &RemoteFilename {
    match self {
      Self::Download { filename, .. } | Self::Upload { filename, .. } => filename,
    }
  }
}

impl Wire for TransferRequest {
  const MIN_BYTES: usize = 12;

  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let direction = r.u32("direction")?;
    let token = r.u32("token")?;
    let filename = r.filename("filename")?;
    match direction {
      0 => Ok(Self::Download { token, filename }),
      1 => Ok(Self::Upload {
        token,
        filename,
        file_size: r.optional_fixed("file_size")?,
      }),
      value => Err(r.fail("direction", FieldError::InvalidValue { value })),
    }
  }

  fn write(&self, w: &mut Writer) {
    w.u32(match self {
      Self::Download { .. } => 0,
      Self::Upload { .. } => 1,
    });
    w.u32(self.token());
    w.filename(self.filename());
    if let Self::Upload {
      file_size: Some(file_size),
      ..
    } = self
    {
      w.u64(*file_size);
    }
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.filename = Some(self.filename().as_str());
    fields.token = Some(self.token());
  }
}

/// Reply to a [`TransferRequest`].
///
/// Peer code 41. Allows the transfer or refuses it with a message.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum TransferResponse {
  /// The transfer may start.
  Allowed {
    token: u32,
    /// Size of the file.
    ///
    /// Present in the deprecated reply to a download request. In the reply to an upload
    /// announcement, some clients send it and others leave it out.
    file_size: Option<u64>,
  },
  /// The transfer does not start now.
  Denied {
    token: u32,
    /// Reason the transfer does not start.
    ///
    /// `"Queued"` means the request was accepted into the queue. Any other message is display
    /// text.
    message: String,
  },
}

impl TransferResponse {
  /// Returns the token of the answered [`TransferRequest`].
  pub fn token(&self) -> u32 {
    match self {
      Self::Allowed { token, .. } | Self::Denied { token, .. } => *token,
    }
  }
}

impl Wire for TransferResponse {
  const MIN_BYTES: usize = 5;

  /// `allowed` is true only for exactly 1, and a denial must carry a message
  fn read(r: &mut Reader, _field: &'static str) -> Result<Self, DecodeError> {
    let token = r.u32("token")?;
    Ok(if r.bool_exact("allowed")? {
      Self::Allowed {
        token,
        file_size: r.optional_fixed("file_size")?,
      }
    } else {
      Self::Denied {
        token,
        message: r.string("message")?,
      }
    })
  }

  fn write(&self, w: &mut Writer) {
    match self {
      Self::Allowed { token, file_size } => {
        w.u32(*token);
        w.bool(true);
        if let Some(file_size) = file_size {
          w.u64(*file_size);
        }
      }
      Self::Denied { token, message } => {
        w.u32(*token);
        w.bool(false);
        w.string(message);
      }
    }
  }

  fn log<'a>(&'a self, _name: &'static str, fields: &mut KeyFields<'a>) {
    fields.token = Some(self.token());
  }
}

wire_struct! {
  /// Message naming a file.
  ///
  /// Peer code 42.
  ///
  /// Obsolete: no client sends this message.
  pub struct UploadPlacehold {
    pub filename: RemoteFilename,
  }
}

wire_struct! {
  /// Request to queue a file for download.
  ///
  /// Peer code 43. Sent to the uploader, to queue the file for the downloader.
  pub struct QueueDownload {
    pub filename: RemoteFilename,
  }
}

wire_struct! {
  /// Place of a download in the upload queue.
  ///
  /// Peer code 44. Sent in reply to a [`PlaceInQueueRequest`], with the place of the downloader in
  /// the uploader's upload queue for a file.
  pub struct PlaceInQueueResponse {
    pub filename: RemoteFilename,
    /// Place in the queue, starting at 1.
    pub place: u32,
  }
}

wire_struct! {
  /// Notification that an upload stopped before it completed.
  ///
  /// Peer code 46. Concerns an upload of a file to the downloader.
  pub struct UploadFailed {
    pub filename: RemoteFilename,
  }
}

wire_struct! {
  /// Refusal or cancellation of an upload, with the reason.
  ///
  /// Peer code 50. Sent by the uploader, for an upload of a file to the downloader.
  pub struct UploadDenied {
    pub filename: RemoteFilename,
    pub reason: String,
  }
}

wire_struct! {
  /// Request for the place of a download in the upload queue.
  ///
  /// Peer code 51. Asks for the downloader's place in the uploader's upload queue for a file.
  pub struct PlaceInQueueRequest {
    pub filename: RemoteFilename,
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::peer::PeerMessage;
  use crate::testing::{decodes, field, p, vectors};

  vectors! {
    transfer_request_download:
      PeerMessage::TransferRequest(TransferRequest::Download { token: 5, filename: "@@share\\a.mp3".into() })
      => p().u32(0).u32(5).s("@@share\\a.mp3").frame(40);
    transfer_request_upload:
      PeerMessage::TransferRequest(TransferRequest::Upload {
        token: 5, filename: "@@share\\a.mp3".into(), file_size: Some(5_242_880),
      })
      => p().u32(1).u32(5).s("@@share\\a.mp3").u64(5_242_880).frame(40);
    transfer_response_allowed_with_size:
      PeerMessage::TransferResponse(TransferResponse::Allowed { token: 5, file_size: Some(5_242_880) })
      => p().u32(5).u8(1).u64(5_242_880).frame(41);
    transfer_response_allowed:
      PeerMessage::TransferResponse(TransferResponse::Allowed { token: 5, file_size: None }) => p().u32(5).u8(1).frame(41);
    transfer_response_denied:
      PeerMessage::TransferResponse(TransferResponse::Denied { token: 5, message: "Queued".into() })
      => p().u32(5).u8(0).s("Queued").frame(41);
    upload_placehold: PeerMessage::UploadPlacehold(UploadPlacehold { filename: "a.mp3".into() }) => p().s("a.mp3").frame(42);
    queue_download:
      PeerMessage::QueueDownload(QueueDownload { filename: "@@share\\a.mp3".into() }) => p().s("@@share\\a.mp3").frame(43);
    queue_download_latin1:
      PeerMessage::QueueDownload(QueueDownload { filename: RemoteFilename::from_bytes(b"caf\xe9\\\xe0.mp3".to_vec()) })
      => p().bytes_string(b"caf\xe9\\\xe0.mp3").frame(43);
    place_in_queue_response:
      PeerMessage::PlaceInQueueResponse(PlaceInQueueResponse { filename: "a.mp3".into(), place: 3 })
      => p().s("a.mp3").u32(3).frame(44);
    upload_failed: PeerMessage::UploadFailed(UploadFailed { filename: "a.mp3".into() }) => p().s("a.mp3").frame(46);
    upload_denied:
      PeerMessage::UploadDenied(UploadDenied { filename: "a.mp3".into(), reason: "File not shared.".into() })
      => p().s("a.mp3").s("File not shared.").frame(50);
    place_in_queue_request:
      PeerMessage::PlaceInQueueRequest(PlaceInQueueRequest { filename: "a.mp3".into() }) => p().s("a.mp3").frame(51);
    upload_queue_notification: PeerMessage::UploadQueueNotification => p().frame(52);
  }

  decodes! {
    transfer_request_download_ignores_a_stray_size: PeerMessage, p().u32(0).u32(5).s("a.mp3").u64(0).frame(40)
      => Ok(PeerMessage::TransferRequest(TransferRequest::Download { token: 5, filename: "a.mp3".into() }));
    transfer_request_upload_without_size: PeerMessage, p().u32(1).u32(5).s("a.mp3").frame(40)
      => Ok(PeerMessage::TransferRequest(TransferRequest::Upload { token: 5, filename: "a.mp3".into(), file_size: None }));
    transfer_request_rejects_an_unknown_direction: PeerMessage, p().u32(2).u32(5).s("a.mp3").frame(40)
      => Err(field("TransferRequest", "direction", FieldError::InvalidValue { value: 2 }));
    transfer_response_allowed_must_be_exactly_one: PeerMessage, p().u32(5).u8(2).s("Cancelled").frame(41)
      => Ok(PeerMessage::TransferResponse(TransferResponse::Denied { token: 5, message: "Cancelled".into() }));
    transfer_response_denied_needs_a_message: PeerMessage, p().u32(5).u8(0).frame(41)
      => Err(field("TransferResponse", "message", FieldError::Truncated { needed: 4, remaining: 0 }));
  }

  #[test]
  fn queue_download_latin1_filename_reads_as_its_text_and_is_lossy() {
    let bytes = p().bytes_string(b"caf\xe9.mp3").frame(43);
    let PeerMessage::QueueDownload(QueueDownload { filename }) =
      PeerMessage::decode(&bytes).unwrap()
    else {
      panic!("not a queue download");
    };
    assert_eq!(filename.as_str(), "caf\u{e9}.mp3");
    assert!(filename.is_lossy());
  }
}
