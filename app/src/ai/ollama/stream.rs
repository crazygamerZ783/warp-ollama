//! Stream adapter for converting OpenAI-compatible SSE responses to
//! `warp_multi_agent_api::ResponseEvent` protobuf events.

use serde::Deserialize;
use uuid::Uuid;
use warp_multi_agent_api as api;

/// A single chunk from the OpenAI-compatible streaming response.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: Option<String>,
    pub choices: Vec<ChunkChoice>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub index: usize,
}

#[derive(Debug, Deserialize)]
pub struct ChunkDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ChunkToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkToolCall {
    #[serde(default)]
    pub index: usize,
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<ChunkFunction>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

/// Generates a `StreamInit` `ResponseEvent` to signal the start of a response.
pub fn create_stream_init_event(request_id: &str) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::Init(
            api::response_event::StreamInit {
                request_id: request_id.to_string(),
                ..Default::default()
            },
        )),
    }
}

/// Converts a text content chunk into a `ClientActions` `ResponseEvent` containing
/// an `AgentOutput` message with the text delta.
pub fn text_chunk_to_response_event(
    text: &str,
    message_id: &str,
    task_id: &str,
) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions {
                messages: vec![api::Message {
                    id: message_id.to_string(),
                    message: Some(api::message::Message::AgentOutput(
                        api::message::AgentOutput {
                            text: text.to_string(),
                        },
                    )),
                    task_id: task_id.to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        )),
    }
}

/// Generates a `StreamFinished` `ResponseEvent` to signal the end of a response.
pub fn create_stream_finished_event(request_id: &str) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::Finished(
            api::response_event::StreamFinished {
                reason: Some(
                    api::response_event::stream_finished::Reason::Done(
                        api::response_event::stream_finished::Done {},
                    ),
                ),
                ..Default::default()
            },
        )),
    }
}

/// Context for tracking state across a streaming response.
pub struct StreamContext {
    pub request_id: String,
    pub task_id: String,
    pub message_id: String,
    pub accumulated_text: String,
}

impl StreamContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            task_id: Uuid::new_v4().to_string(),
            message_id: Uuid::new_v4().to_string(),
            accumulated_text: String::new(),
        }
    }
}
