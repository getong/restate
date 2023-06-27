//! Module containing definitions of Protocol messages,
//! including encoding and decoding of headers and message payloads.

use super::pb;

use bytes::Bytes;
use prost::Message;
use restate_common::errors::InvocationError;
use restate_common::journal::raw::PlainRawEntry;
use restate_common::journal::Completion;
use restate_common::types::CompletionResult;

mod encoding;
mod header;

pub use encoding::{Decoder, Encoder, EncodingError};
pub use header::{MessageHeader, MessageKind, MessageType};

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolMessage {
    // Core
    Start {
        partial_state: bool,
        inner: pb::protocol::StartMessage,
    },
    Completion(pb::protocol::CompletionMessage),
    Suspension(pb::protocol::SuspensionMessage),
    Error(pb::protocol::ErrorMessage),

    // Entries are not parsed at this point
    UnparsedEntry(PlainRawEntry),
}

impl ProtocolMessage {
    pub fn new_start_message(
        invocation_id: Bytes,
        instance_key: Bytes,
        known_entries: u32,
        partial_state: bool,
        state_map_entries: impl IntoIterator<Item = (Bytes, Bytes)>,
    ) -> Self {
        Self::Start {
            partial_state,
            inner: pb::protocol::StartMessage {
                invocation_id,
                instance_key,
                known_entries,
                state_map: state_map_entries
                    .into_iter()
                    .map(|(key, value)| pb::protocol::start_message::StateEntry { key, value })
                    .collect(),
            },
        }
    }
}

impl From<Completion> for ProtocolMessage {
    fn from(completion: Completion) -> Self {
        ProtocolMessage::Completion(pb::protocol::CompletionMessage {
            entry_index: completion.entry_index,
            result: match completion.result {
                CompletionResult::Ack => None,
                CompletionResult::Empty => {
                    Some(pb::protocol::completion_message::Result::Empty(()))
                }
                CompletionResult::Success(b) => {
                    Some(pb::protocol::completion_message::Result::Value(b))
                }
                CompletionResult::Failure(code, message) => Some(
                    pb::protocol::completion_message::Result::Failure(pb::protocol::Failure {
                        code: code.into(),
                        message: message.to_string(),
                    }),
                ),
            },
        })
    }
}

impl From<PlainRawEntry> for ProtocolMessage {
    fn from(value: PlainRawEntry) -> Self {
        Self::UnparsedEntry(value)
    }
}

impl From<pb::protocol::ErrorMessage> for InvocationError {
    fn from(value: pb::protocol::ErrorMessage) -> Self {
        if value.description.is_empty() {
            InvocationError::new(value.code, value.message)
        } else {
            InvocationError::new_with_description(value.code, value.message, value.description)
        }
    }
}
