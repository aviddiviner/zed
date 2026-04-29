# Zed Gated Features — Notes for Enabling

These are features present in Zed's open-source code but gated behind their cloud service or feature flags. To enable them independently in a vanilla Zed build (or Aleph):

## WebSearchTool

**What it does:** Lets the agent search the web for real-time information.

**How it's gated:**
- `crates/agent/src/tools/web_search_tool.rs` — `supports_provider()` returns `true` only for `ZED_CLOUD_PROVIDER_ID`
- `crates/web_search_providers/src/web_search_providers.rs` — only registers a search provider when the active LLM is Zed's cloud provider

**To enable independently:**
1. Change `supports_provider` to return `true` for all providers
2. Register a web search provider that doesn't depend on Zed's cloud. Options:
   - Brave Search API (free tier available, simple REST API)
   - SearXNG (self-hosted, no API key needed)
   - Google Custom Search API
3. Implement the `WebSearchProvider` trait (see `crates/web_search/src/`) with your chosen backend
4. Register it in a new init function (or modify `web_search_providers::init`)
5. Call that init from `main.rs`

**Key files:**
- `crates/web_search/src/` — trait definition
- `crates/web_search_providers/src/` — provider implementations
- `crates/agent/src/tools/web_search_tool.rs` — the tool itself

## UpdatePlanTool

**What it does:** Tracks multi-step tasks with a visible plan UI (checkboxes, progress).

**How it's gated:**
- `crates/agent/src/thread.rs` — only added when `cx.has_flag::<UpdatePlanToolFeatureFlag>()`
- Feature flags are typically synced from Zed's server (`feature_flags` crate)

**To enable independently:**
1. Remove the `if cx.has_flag::<UpdatePlanToolFeatureFlag>()` check
2. Just call `self.add_tool(UpdatePlanTool)` unconditionally
3. That's it — the tool itself has no cloud dependencies, it's purely local UI

**Key files:**
- `crates/agent/src/thread.rs` (~line 1562) — where it's conditionally added
- `crates/agent/src/tools/update_plan_tool.rs` — the tool implementation