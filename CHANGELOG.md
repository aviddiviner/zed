# Changelog

A narrative log of Aleph's development, grouped by session. Captures what was done and why — context that commit messages alone don't convey.

---

## 2026-05-17 — Deep crate removal (always-local architecture)

Systematic removal of dead crates from the dependency tree. Not just "removed from init" (Phase 1 approach) but fully deleted from disk, including all integration points in downstream crates.

**Deleted crates (21 + 6 focused removals):**

- 21 dead crates in one sweep: `vim`, `edit_prediction_metrics`, `audio`, `denoise`, `component_preview`, `input_latency_ui`, `inspector_ui`, `miniprofiler_ui`, `docs_preprocessor`, `schema_generator`, `extension_cli`, `theme_importer`, `fs_benchmarks`, `project_benchmarks`, `worktree_benchmarks`, `explorer_command_injector`, `etw_tracing`, `gpui_web`, `nc`, `csv_preview`, `svg_preview`
- `vim_mode_setting` — removed from editor (change-list tracking now unconditional) and extensions_ui
- `channel` — stripped from file_finder (channel search results) and notifications (channel invitations)
- `dev_container` — stripped from recent_projects (suggestion UI, picker, remote server connection)
- `dap` — stripped from editor (breakpoints, debug lines, inline values), project (entire debugger module, breakpoint store, dap store, sessions, locators), workspace (breakpoint persistence), extension/extension_host (debug adapter types), json_schema_store (debug task schemas)
- `remote` + `remote_connection` + `auto_update` — made Project always-local. Removed remote constructors, ProjectClientState enum, remote_client field, connection state, remote environment resolution. Stripped remote connection UI from agent_ui, sidebar, git_ui, recent_projects. Removed extension syncing to remotes.

**Also:** Removed Python from built-in languages. Stripped `libwebrtc`/`livekit` patches. Removed `gpui_web` conditional deps from gpui/gpui_platform.

**Net result:** ~67,000 lines deleted. The editor no longer knows about breakpoints, debug adapters, or remote connections. The project is always local.

---

## 2026-05-15 — MCP resource fix

Fixed MCP resource content type not being surfaced to model. Resources with explicit content types were being ignored during context assembly.

---

## 2026-05-10 — Merge safeguards and About window

- Added RELEASE_CHANNEL safeguards (`.gitattributes` merge=ours, build script check) after discovering it resets to "stable" during upstream merges, breaking credential storage and MCP startup.
- Added About window showing the Zed base version and current git commit.

---

## 2026-05-09 — Merge Zed v1.1.7

Upstream sync. Brings bug fixes and stability improvements. Updated Claude model token limits for Bedrock.

---

## 2026-05-06 — MCP protocol update

Added support for MCP protocol version 2025-06-18.

---

## 2026-05-04 — Stability fixes + Merge Zed v1.0.1

- Fixed cmd+f search and cmd+, settings not working reliably (action dispatch issue)
- Relaxed OAuth issuer validation to support parent-domain issuers (needed for some MCP auth flows)
- Stripped breakpoints from editor gutter entirely, simplified bookmark UX
- Merged Zed v1.0.1 (bug fixes)

---

## 2026-05-02 — Merge Zed v1.0.0 + major cleanup

Merged Zed v1.0.0 for model updates (Opus 4.7, GPT-5.5, DeepSeek v4), performance fixes, markdown heading sizes, git improvements, and bookmarks.

Post-merge work:
- Disabled all telemetry and prevented phoning home to zed.dev
- Switched to file-based credentials provider (no system keychain)
- Bookmarks enabled in gutter (breakpoints/runnables disabled)
- Per-tool enable/disable support for MCP servers
- Removed `call`/`livekit` crates, `ai_onboarding` crate, Copilot provider
- Renamed `zed_agent` → `aleph_agent`, `zed_logo` → `aleph_logo`
- Removed 22 dead icon SVGs and 14 unused enum variants
- Removed 12 unused crate directories
- Added `keymap_editor` crate
- Changed log paths from Zed to Aleph
- Fixed shell environment capture (added `--printenv` CLI flag)
- Stripped `languages` crate to writing-relevant languages only

---

## 2026-04-30 — Polish and quality of life

- Enabled `UpdatePlanTool` unconditionally (was gated behind Zed premium)
- Replaced agent icon with geometric aleph letterform
- Replaced Zed logo with Aleph logo throughout
- Agent's `delete_path` tool now sends to Trash instead of permanent delete
- Added writing toolbar with markdown preview button
- Relaxed auth server metadata validation

---

## 2026-04-29 — Initial fork (Phase 1)

Created Aleph from Zed v0.233.10. In one day:

**Binary and identity:**
- Created `crates/aleph` binary crate, stripped dependencies and init code
- Rebranded menus, title bar, welcome page, agent panel to "Aleph"
- Separate config (`~/.config/aleph/`) and data (`~/Library/Application Support/Aleph/`) directories
- Removed Zed cloud LLM provider and auto-authenticate

**Writing defaults:**
- Soft wrap, comfortable line height, no line numbers, no wrap guides
- Rewrote agent system prompt for writing collaborator persona
- Rewrote agent templates (no more "expert engineer")
- Removed DiagnosticsTool and TerminalTool from agent tools

**Configuration:**
- Configurable system prompt via `~/.config/aleph/system_prompt.hbs` with hot-reload
- User theme loading from `~/.config/aleph/themes/` with hot-reload
- Bundled "Aleph Latte" theme as default
- App icon (SF Pro ℵ on warm cream background)
