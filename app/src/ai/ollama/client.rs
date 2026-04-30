//! HTTP client for communicating with a local Ollama instance via the
//! OpenAI-compatible `/v1/chat/completions` endpoint.

use std::sync::Arc;

use futures::channel::oneshot;
use futures::stream::{self, StreamExt};
use warp_multi_agent_api as api;

use crate::ai::agent::api::{Event, ResponseStream};
use crate::server::server_api::AIApiError;

use super::config::OllamaConfig;
use super::stream::{
    create_stream_finished_event, create_stream_init_event, text_chunk_to_response_event,
    ChatCompletionChunk, StreamContext,
};

/// An HTTP client that sends requests to a local Ollama instance and returns
/// a stream of `warp_multi_agent_api::ResponseEvent`s compatible with the rest
/// of the Warp AI pipeline.
pub struct OllamaClient;

impl OllamaClient {
    /// Checks if the given LLMId refers to an Ollama model.
    pub fn is_ollama_model(model_id: &str) -> bool {
        OllamaConfig::is_ollama_model_id(model_id)
    }

    /// Sends a request to the local Ollama instance and returns a stream of
    /// `ResponseEvent`s that the Warp conversation pipeline can consume.
    ///
    /// This converts the `warp_multi_agent_api::Request` into an OpenAI-compatible
    /// chat completions request, streams the SSE response, and converts each chunk
    /// back into `ResponseEvent` protobuf events.
    pub async fn generate_output(
        request: &api::Request,
        cancellation_rx: oneshot::Receiver<()>,
    ) -> ResponseStream {
        let config = OllamaConfig::global();
        let ctx = StreamContext::new();

        // Extract user messages from the request to build OpenAI-compatible messages
        let messages = extract_messages_from_request(request);
        let model_name = extract_model_name(request).unwrap_or_else(|| config.model.clone());

        // Build the OpenAI-compatible request body
        let body = serde_json::json!({
            "model": model_name,
            "messages": messages,
            "stream": true,
        });

        let url = config.chat_completions_url();
        let client = reqwest::Client::new();

        // Send the request
        let response = match client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("Failed to connect to Ollama at {}: {}", url, e);
                // Return a stream with just an error event
                let error_events: Vec<Event> = vec![
                    Ok(create_stream_init_event(&ctx.request_id)),
                    Ok(text_chunk_to_response_event(
                        &format!(
                            "Error: Could not connect to Ollama at {}. \
                            Make sure Ollama is running (`ollama serve`).\n\nDetails: {}",
                            url, e
                        ),
                        &ctx.message_id,
                        &ctx.task_id,
                    )),
                    Ok(create_stream_finished_event(&ctx.request_id)),
                ];
                return Box::pin(stream::iter(error_events));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            log::error!(
                "Ollama returned error status {}: {}",
                status,
                body_text
            );
            let error_events: Vec<Event> = vec![
                Ok(create_stream_init_event(&ctx.request_id)),
                Ok(text_chunk_to_response_event(
                    &format!(
                        "Error: Ollama returned status {}.\n\nDetails: {}",
                        status, body_text
                    ),
                    &ctx.message_id,
                    &ctx.task_id,
                )),
                Ok(create_stream_finished_event(&ctx.request_id)),
            ];
            return Box::pin(stream::iter(error_events));
        }

        // Parse the SSE stream
        let request_id = ctx.request_id.clone();
        let message_id = ctx.message_id.clone();
        let task_id = ctx.task_id.clone();

        // Create init event
        let init_event: Event = Ok(create_stream_init_event(&request_id));

        // Read the streaming body line-by-line and convert chunks
        let byte_stream = response.bytes_stream();

        let chunk_stream = byte_stream
            .filter_map({
                let message_id = message_id.clone();
                let task_id = task_id.clone();
                let mut buffer = String::new();
                move |chunk_result| {
                    let message_id = message_id.clone();
                    let task_id = task_id.clone();
                    let events = match chunk_result {
                        Ok(bytes) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));
                            let mut events = Vec::new();

                            // Process complete lines from the buffer
                            while let Some(newline_pos) = buffer.find('\n') {
                                let line = buffer[..newline_pos].trim().to_string();
                                buffer = buffer[newline_pos + 1..].to_string();

                                if line.is_empty() {
                                    continue;
                                }

                                // Handle SSE "data:" prefix
                                let data = if let Some(stripped) = line.strip_prefix("data: ") {
                                    stripped.trim()
                                } else if let Some(stripped) = line.strip_prefix("data:") {
                                    stripped.trim()
                                } else {
                                    continue;
                                };

                                // "[DONE]" signals end of stream
                                if data == "[DONE]" {
                                    continue;
                                }

                                // Parse the JSON chunk
                                match serde_json::from_str::<ChatCompletionChunk>(data) {
                                    Ok(chunk) => {
                                        for choice in &chunk.choices {
                                            if let Some(content) = &choice.delta.content {
                                                if !content.is_empty() {
                                                    events.push(Ok(text_chunk_to_response_event(
                                                        content,
                                                        &message_id,
                                                        &task_id,
                                                    )));
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to parse Ollama SSE chunk: {} — raw: {}",
                                            e,
                                            data
                                        );
                                    }
                                }
                            }

                            events
                        }
                        Err(e) => {
                            log::error!("Error reading Ollama stream: {}", e);
                            vec![Err(Arc::new(AIApiError::Other(e.into())))]
                        }
                    };

                    // Yield each event from the vec
                    async move {
                        if events.is_empty() {
                            None
                        } else {
                            Some(stream::iter(events))
                        }
                    }
                }
            })
            .flatten();

        // Build the complete stream: init → content chunks → finished
        let finished_event: Event = Ok(create_stream_finished_event(&request_id));

        let full_stream = stream::once(async { init_event })
            .chain(chunk_stream)
            .chain(stream::once(async { finished_event }));

        // Apply cancellation
        let cancellable_stream = full_stream.take_until(cancellation_rx);

        Box::pin(cancellable_stream)
    }
}

/// Extracts chat messages from a `warp_multi_agent_api::Request` in OpenAI format.
///
/// Converts the internal input format to a JSON array of `{"role": ..., "content": ...}` objects.
fn extract_messages_from_request(request: &api::Request) -> Vec<serde_json::Value> {
    let mut messages = Vec::new();

    // Add a system message for context
    messages.push(serde_json::json!({
        "role": "system",
        "content": "You are a helpful AI assistant integrated into the Warp terminal. \
                    Help the user with coding tasks, command-line operations, and technical questions. \
                    When suggesting shell commands, format them as code blocks. \
                    Be concise and practical."
    }));

    // Extract user input from the request
    if let Some(input) = &request.input {
        match &input.r#type {
            Some(api::request::input::Type::GenerateOutput(generate_output)) => {
                // Extract messages from the conversation history
                for message in &generate_output.messages {
                    if let Some(msg) = &message.message {
                        match msg {
                            api::message::Message::UserQuery(query) => {
                                messages.push(serde_json::json!({
                                    "role": "user",
                                    "content": query.query,
                                }));
                            }
                            api::message::Message::AgentOutput(output) => {
                                messages.push(serde_json::json!({
                                    "role": "assistant",
                                    "content": output.text,
                                }));
                            }
                            // Skip other message types for now
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // If we only have the system message, the request probably has no conversation
    // history yet — just pass through with a generic user message placeholder
    if messages.len() == 1 {
        // Try to extract the raw query from any available input context
        if let Some(input) = &request.input {
            match &input.r#type {
                Some(api::request::input::Type::GenerateOutput(gen)) => {
                    for msg in &gen.messages {
                        if let Some(api::message::Message::UserQuery(q)) =
                            msg.message.as_ref()
                        {
                            if !q.query.is_empty() {
                                messages.push(serde_json::json!({
                                    "role": "user",
                                    "content": q.query,
                                }));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    messages
}

/// Extracts the model name from the request settings, stripping the "ollama:" prefix.
fn extract_model_name(request: &api::Request) -> Option<String> {
    request
        .settings
        .as_ref()
        .and_then(|s| s.model_config.as_ref())
        .map(|mc| &mc.base)
        .filter(|id| !id.is_empty())
        .and_then(|id| OllamaConfig::extract_model_name(id))
        .map(|s| s.to_string())
}
