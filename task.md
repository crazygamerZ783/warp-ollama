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
- [x] Modify `LLMPreferences::new()` for offline model list
- [x] Bypass auth gates in AI UI rendering
- [x] Allow AI features without sign-in when Ollama enabled

## Phase 4: Settings UI
- [x] Add Ollama configuration section to AI settings page

## Phase 5: Build Verification
- [x] Attempt cargo build (Pending Rust toolchain installation -> Resolved)
- [x] Fix compilation errors

## Phase 6: Fix Offline Streaming Errors
- [x] Extract exact `task_id` from the request
- [x] Use `AddMessagesToTask` to initialize the message structure for the very first chunk
- [x] Use `AppendToMessageContent` for all subsequent chunks

## Phase 7: Unit Testing
- [x] Write tests for `text_chunk_to_response_event`
- [x] Run `cargo test` and verify tests pass

# Status Update
- Core Ollama client and streaming infrastructure implemented in `app/src/ai/ollama/`.
- Provider routing updated in `app/src/ai/llms.rs` and `app/src/server/server_api/ai.rs`.
- Auth gate bypass implemented in `app/src/settings/ai.rs`.
- Local model injection into UI selectors implemented in `app/src/ai/llms.rs`.
- Settings UI for Ollama configuration added to `app/src/settings_view/ai_page.rs`.
- Integration into the main AI module completed in `app/src/ai/mod.rs`.
- Fixed offline conversation state initialization (`ExchangeNotFound` and `TaskNotFound` errors).
- Unit tests added to verify stream conversion logic.
- Compilation and tests successful! The code is ready for testing.
