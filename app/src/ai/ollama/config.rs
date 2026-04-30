//! Configuration for the local Ollama endpoint.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use parking_lot::RwLock;

/// Default Ollama API endpoint.
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Default model to use with Ollama.
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.1";

/// Prefix used to identify Ollama model IDs in the LLMId system.
/// An Ollama model LLMId looks like "ollama:llama3.1".
pub const OLLAMA_MODEL_PREFIX: &str = "ollama:";

/// Global Ollama configuration singleton.
static OLLAMA_CONFIG: OnceLock<RwLock<OllamaConfig>> = OnceLock::new();

/// Configuration for connecting to a local Ollama instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Whether Ollama integration is enabled.
    pub enabled: bool,
    /// The base URL for the Ollama API (e.g., "http://localhost:11434").
    pub base_url: String,
    /// The model name to use (e.g., "llama3.1", "codellama", "mistral").
    pub model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: DEFAULT_OLLAMA_URL.to_string(),
            model: DEFAULT_OLLAMA_MODEL.to_string(),
        }
    }
}

impl OllamaConfig {
    /// Returns the OpenAI-compatible chat completions endpoint URL.
    pub fn chat_completions_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'))
    }

    /// Returns the Ollama API tags endpoint for listing available models.
    pub fn tags_url(&self) -> String {
        format!("{}/api/tags", self.base_url.trim_end_matches('/'))
    }

    /// Creates an LLMId string for a given model name.
    pub fn model_id(model_name: &str) -> String {
        format!("{}{}", OLLAMA_MODEL_PREFIX, model_name)
    }

    /// Checks if an LLMId string represents an Ollama model.
    pub fn is_ollama_model_id(id: &str) -> bool {
        id.starts_with(OLLAMA_MODEL_PREFIX)
    }

    /// Extracts the model name from an Ollama LLMId.
    pub fn extract_model_name(id: &str) -> Option<&str> {
        id.strip_prefix(OLLAMA_MODEL_PREFIX)
    }

    /// Gets the global Ollama configuration.
    pub fn global() -> OllamaConfig {
        OLLAMA_CONFIG
            .get_or_init(|| RwLock::new(OllamaConfig::default()))
            .read()
            .clone()
    }

    /// Updates the global Ollama configuration.
    pub fn set_global(config: OllamaConfig) {
        let lock = OLLAMA_CONFIG.get_or_init(|| RwLock::new(OllamaConfig::default()));
        *lock.write() = config;
    }
}
