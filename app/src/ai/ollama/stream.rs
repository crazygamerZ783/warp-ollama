//! Stream adapter for converting OpenAI-compatible SSE responses to
//! `warp_multi_agent_api::ResponseEvent` protobuf events.

use serde::Deserialize;
use uuid::Uuid;
use warp_multi_agent_api as api;

/// A single chunk from the OpenAI-compatible streaming response.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    pub delta: ChunkDelta,
}

#[derive(Debug, Deserialize)]
pub struct ChunkDelta {
    #[serde(default)]
    pub content: Option<String>,
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
/// an `AppendToMessageContent` action with the text delta.
pub fn text_chunk_to_response_event(
    text: &str,
    message_id: &str,
    task_id: &str,
    is_first_chunk: bool,
) -> api::ResponseEvent {
    let mut actions = Vec::new();

    // If it's the first chunk, wrap it in a transaction and initialize the message
    if is_first_chunk {
        actions.push(api::ClientAction {
            action: Some(api::client_action::Action::BeginTransaction(
                api::client_action::BeginTransaction {},
            )),
        });
        
        actions.push(api::ClientAction {
            action: Some(api::client_action::Action::AddMessagesToTask(
                api::client_action::AddMessagesToTask {
                    task_id: task_id.to_string(),
                    messages: vec![api::Message {
                        id: message_id.to_string(),
                        message: Some(api::message::Message::AgentOutput(
                            api::message::AgentOutput {
                                text: String::new(),
                            },
                        )),
                        task_id: task_id.to_string(),
                        ..Default::default()
                    }],
                },
            )),
        });

        actions.push(api::ClientAction {
            action: Some(api::client_action::Action::CommitTransaction(
                api::client_action::CommitTransaction {},
            )),
        });
    }

    actions.push(api::ClientAction {
        action: Some(api::client_action::Action::AppendToMessageContent(
            api::client_action::AppendToMessageContent {
                task_id: task_id.to_string(),
                message: Some(api::Message {
                    id: message_id.to_string(),
                    message: Some(api::message::Message::AgentOutput(
                        api::message::AgentOutput {
                            text: text.to_string(),
                        },
                    )),
                    task_id: task_id.to_string(),
                    ..Default::default()
                }),
                mask: Some(prost_types::FieldMask {
                    paths: vec!["message.agent_output.text".to_string()],
                }),
            },
        )),
    });

    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions { actions },
        )),
    }
}

/// Generates a `StreamFinished` `ResponseEvent` to signal the end of a response.
pub fn create_stream_finished_event(_request_id: &str) -> api::ResponseEvent {
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
}

impl StreamContext {
    pub fn new(task_id: String) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            task_id,
            message_id: Uuid::new_v4().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp_multi_agent_api::client_action::Action;

    #[test]
    fn test_text_chunk_to_response_event_first_chunk() {
        let text = "Hello";
        let message_id = "msg-123";
        let task_id = "task-456";
        let is_first_chunk = true;

        let event = text_chunk_to_response_event(text, message_id, task_id, is_first_chunk);

        if let Some(api::response_event::Type::ClientActions(client_actions)) = &event.r#type {
            let actions = &client_actions.actions;
            assert_eq!(actions.len(), 4);

            assert!(matches!(
                actions[0].action,
                Some(Action::BeginTransaction(_))
            ));

            if let Some(Action::AddMessagesToTask(add_msg)) = &actions[1].action {
                assert_eq!(add_msg.task_id, task_id);
                assert_eq!(add_msg.messages.len(), 1);
                assert_eq!(add_msg.messages[0].id, message_id);
            } else {
                panic!("Expected AddMessagesToTask action");
            }

            assert!(matches!(
                actions[2].action,
                Some(Action::CommitTransaction(_))
            ));

            if let Some(Action::AppendToMessageContent(append)) = &actions[3].action {
                assert_eq!(append.task_id, task_id);
                let msg = append.message.as_ref().unwrap();
                assert_eq!(msg.id, message_id);
                if let Some(api::message::Message::AgentOutput(output)) = &msg.message {
                    assert_eq!(output.text, text);
                } else {
                    panic!("Expected AgentOutput message");
                }
            } else {
                panic!("Expected AppendToMessageContent action");
            }
        } else {
            panic!("Expected ClientActions response event");
        }
    }

    #[test]
    fn test_text_chunk_to_response_event_subsequent_chunk() {
        let text = " World";
        let message_id = "msg-123";
        let task_id = "task-456";
        let is_first_chunk = false;

        let event = text_chunk_to_response_event(text, message_id, task_id, is_first_chunk);

        if let Some(api::response_event::Type::ClientActions(client_actions)) = &event.r#type {
            let actions = &client_actions.actions;
            assert_eq!(actions.len(), 1);

            if let Some(Action::AppendToMessageContent(append)) = &actions[0].action {
                assert_eq!(append.task_id, task_id);
                let msg = append.message.as_ref().unwrap();
                assert_eq!(msg.id, message_id);
                if let Some(api::message::Message::AgentOutput(output)) = &msg.message {
                    assert_eq!(output.text, text);
                } else {
                    panic!("Expected AgentOutput message");
                }
            } else {
                panic!("Expected AppendToMessageContent action");
            }
        } else {
            panic!("Expected ClientActions response event");
        }
    }
}
