# Ollama Support for Warp Terminal AI Features

Add support for local Ollama models as an AI provider, using its OpenAI-compatible API endpoint, and allow AI features to work without sign-in.

## User Review Required

> [!IMPORTANT]
> **This is a very large, high-risk modification to a commercial codebase.** The Warp terminal AI pipeline is deeply integrated with server-side authentication (OAuth tokens), server-side model routing, server-side billing/quota tracking, and a protobuf-based streaming protocol (`warp_multi_agent_api`). The AI requests go to Warp's server (`/ai/multi-agent`), which proxies to OpenAI/Anthropic/Google — the client never talks to LLM APIs directly.

> [!CAUTION]
> **Fundamental architecture mismatch:** Warp's AI features are server-mediated. The client sends protobuf-encoded requests to `{server_root_url}/ai/multi-agent` via SSE, which Warp's server decodes, routes to the appropriate LLM provider, and streams back protobuf-encoded `ResponseEvent`s. Adding Ollama means we need to **bypass the entire Warp server pipeline** and talk directly to a local Ollama instance, converting between the OpenAI chat completions format and Warp's internal `warp_multi_agent_api::ResponseEvent` protobuf format. This is a massive undertaking.

> [!WARNING]
> **Auth bypass scope:** Making AI work "without sign-in" requires changes to every place that gates on `is_logged_in()` or requires an auth token. The current flow is: auth token → server validates → routes request. Without auth, we skip the server entirely and go local-only.

## Open Questions

> [!IMPORTANT]
> 1. **Which AI features should work offline?** The full Agent Mode pipeline uses ~15 tool types (ReadFiles, ApplyFileDiffs, RunShellCommand, Grep, etc.) that are executed client-side but *orchestrated* server-side. To make Agent Mode work with Ollama, we'd need to re-implement the entire tool-call orchestration loop locally. The simpler features (command generation, dialogue) are also server-mediated. **Do you want ALL features or just the basic chat/agent conversation?**

> [!IMPORTANT]
> 2. **Ollama endpoint configuration:** Where should the Ollama URL (default `http://localhost:11434`) and model name be configured? Options:
>    - In the existing AI Settings page alongside the BYO API key settings
>    - In a new "Local Models" settings section
>    - Via environment variables only

> [!IMPORTANT]
> 3. **Feature parity expectations:** Ollama models won't support many Warp-specific features like:
>    - Tool calling (unless the model supports it — e.g., `llama3.1` does, `codellama` doesn't)
>    - Code embeddings / codebase indexing
>    - Memory / rules system (server-mediated)
>    - Computer use agent
>    - Research agent / orchestration
>    - File artifacts / attachments
>    - All server-mediated features (quota tracking, conversation persistence, etc.)
>    
>    **Is a basic text-only chat + simple command suggestions acceptable as v1?**

> [!WARNING]
> 4. **Compilation feasibility on Windows:** Warp is a massive Rust monorepo with 100+ crates, platform-specific GPU rendering code, and complex build dependencies (protobuf, system libraries, etc.). Have you successfully built this codebase before? The compilation alone requires significant setup. I want to set expectations that getting it to compile may be a separate multi-hour effort from the code changes.

## Proposed Changes

Given the massive scope, I propose a **phased approach** starting with the minimum viable change:

### Phase 1: Ollama Provider + Local AI Client (Core Infrastructure)

#### [NEW] `app/src/ai/ollama/mod.rs`
- New module for Ollama integration
- `OllamaClient` struct with configurable endpoint URL and model name
- HTTP client for OpenAI-compatible `/v1/chat/completions` endpoint
- SSE streaming response parser
- Conversion from OpenAI chat completion chunks → `warp_multi_agent_api::ResponseEvent` protobuf events

#### [NEW] `app/src/ai/ollama/config.rs`
- `OllamaConfig` struct (endpoint URL, model name, enabled flag)
- Persistence via user preferences
- Default values (`http://localhost:11434`, model auto-detect)

#### [NEW] `app/src/ai/ollama/stream.rs`
- OpenAI-compatible SSE stream adapter
- Converts `data: {"choices":[{"delta":{"content":"..."}}]}` → `ResponseEvent` protobuf messages
- Handles tool calls if the model supports them

---

### Phase 2: Provider Routing + Auth Bypass

#### [MODIFY] `app/src/ai/llms.rs`
- Add `LLMProvider::Ollama` variant
- Add Ollama models to `ModelsByFeature` when Ollama is configured
- Bypass server fetch for model list when using Ollama

#### [MODIFY] `app/src/server/server_api.rs` — `generate_multi_agent_output()`
- Route to `OllamaClient` when the selected model is an Ollama model
- Skip auth token requirement for Ollama requests
- Return local SSE stream instead of server stream

#### [MODIFY] `app/src/ai/agent/api/impl.rs` — `generate_multi_agent_output()`
- Check if selected model is Ollama and use local client
- Skip server_api call for local models

#### [MODIFY] `crates/ai/src/api_keys.rs`
- No changes needed — Ollama doesn't use API keys

---

### Phase 3: Auth Gate Bypass for Offline Mode

#### [MODIFY] `app/src/ai/llms.rs` — `LLMPreferences::new()`
- Don't require auth for model list refresh when Ollama is configured
- Provide local fallback `ModelsByFeature` with Ollama models

#### [MODIFY] Multiple files that check `is_logged_in()` before showing AI UI:
- `app/src/terminal/view.rs` — Auth gate for agent input
- `app/src/workspace/view.rs` — Auth gate for AI features
- `app/src/settings_view/ai_page.rs` — Show Ollama config without login

---

### Phase 4: Settings UI

#### [MODIFY] `app/src/settings_view/ai_page.rs`
- Add "Local Models (Ollama)" section
- Toggle to enable/disable Ollama
- Text input for endpoint URL
- Model name selector (query Ollama `/api/tags` for available models)

---

### Phase 5: Build & Compile

#### Build system verification
- Verify all Rust dependencies resolve on Windows
- Verify protobuf compilation works
- Run `cargo build` for the `app` target

## Verification Plan

### Automated Tests
- Unit tests for `OllamaClient` stream parsing
- Unit tests for OpenAI → ResponseEvent conversion
- Integration test with a mock Ollama server

### Manual Verification
- Run Ollama locally with `ollama run llama3.1`
- Open Warp without signing in
- Verify AI chat sends to local Ollama and receives streamed responses
- Verify model selector shows Ollama models
- Verify settings page allows configuring endpoint

## Estimated Effort

| Phase | Effort | Risk |
|-------|--------|------|
| Phase 1: Ollama Client | High (~500-800 lines) | Medium — format conversion is complex |
| Phase 2: Provider Routing | High (~200-400 lines) | **High** — core request pipeline changes |
| Phase 3: Auth Bypass | Medium (~100-200 lines) | **High** — many call sites to audit |
| Phase 4: Settings UI | Medium (~200-300 lines) | Low — follows existing patterns |
| Phase 5: Build | Variable | **Very High** — depends on environment |

**Total: ~1000-1700 lines of Rust code changes across 15+ files**
