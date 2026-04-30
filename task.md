# Ollama Support Implementation Tasks

## Phase 1: Ollama Client Infrastructure
- [x] Research `warp_multi_agent_api::ResponseEvent` protobuf format
- [x] Research how the stream is consumed in conversation model
- [x] Create `app/src/ai/ollama/mod.rs` — module declaration
- [x] Create `app/src/ai/ollama/config.rs` — Ollama config struct
- [x] Create `app/src/ai/ollama/client.rs` — OllamaClient implementation
- [x] Create `app/src/ai/ollama/stream.rs` — SSE stream adapter
- [x] Register `ollama` module in `app/src/ai/mod.rs`

## Phase 2: Provider Routing
- [x] Add `LLMProvider::Ollama` to `app/src/ai/llms.rs`
- [x] Add Ollama models to `ModelsByFeature` defaults
- [x] Modify `generate_multi_agent_output` to route to Ollama
- [x] Skip server auth for Ollama requests

## Phase 3: Auth Gate Bypass
- [ ] Modify `LLMPreferences::new()` for offline model list
- [ ] Bypass auth gates in AI UI rendering
- [ ] Allow AI features without sign-in when Ollama enabled

## Phase 4: Settings UI
- [ ] Add Ollama configuration section to AI settings page

## Phase 5: Build Verification
- [ ] Attempt cargo build
- [ ] Fix compilation errors
