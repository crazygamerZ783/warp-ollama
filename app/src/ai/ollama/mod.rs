//! Ollama integration module for local LLM support.
//!
//! This module provides an OpenAI-compatible client that talks to a local Ollama
//! instance, converting between the OpenAI chat completions streaming format and
//! Warp's internal `warp_multi_agent_api::ResponseEvent` protobuf protocol.

pub mod client;
pub mod config;
pub mod stream;

pub use client::OllamaClient;
