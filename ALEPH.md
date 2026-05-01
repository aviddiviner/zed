# ℵ Aleph — Agentic Writing Environment

> A fork of [Zed](https://zed.dev) re-imagined as a creative writing tool with first-class AI agent support.

## Vision

Aleph takes the blazing-fast editor, beautiful UI framework (GPUI), and production-grade AI agent infrastructure from Zed and strips away the code-specific tooling, replacing it with a purpose-built environment for fiction and non-fiction writers.

The "Agentic" part means your AI assistant isn't just autocomplete — it's a full collaborator that can read your manuscript, suggest prose improvements, catch continuity errors, research settings, track characters, and help you through the writing process.

---

## Architecture (inherited from Zed)

```
┌─────────────────────────────────────────────────────┐
│                    Aleph Binary                     │
├─────────────────────────────────────────────────────┤
│  Workspace  │  Editor  │  Agent UI  │  Panels       │
├─────────────────────────────────────────────────────┤
│  GPUI (UI Framework + Rendering + Layout)           │
├─────────────────────────────────────────────────────┤
│  text/rope │ language (tree-sitter) │ fs/worktree   │
└─────────────────────────────────────────────────────┘
```

---

## Phases

### Phase 1A: Fork & Strip ✅ COMPLETE

**Goal:** A building, running app with code-specific features removed.

#### Steps:

- [x] Create `crates/aleph` binary crate (fork of `crates/zed`)
- [x] Strip dependencies from `aleph/Cargo.toml`
- [x] Strip initialization code in `main.rs` — don't init removed features
- [x] Update workspace Cargo.toml to include `crates/aleph`
- [x] Fix keymap loading to gracefully skip unknown actions (`load_asset_allow_partial_failure`)
- [x] Add `RefreshLlmTokenListener::register` for LLM infrastructure
- [x] Confirm it builds and opens a window with: editor + file tree + agent panel

#### Removal List (confirmed by human review):

| Group | Crates | Status |
|-------|--------|--------|
| Debugger & Tasks | `dap`, `dap_adapters`, `debugger_ui`, `debugger_tools`, `debug_adapter_extension`, `task`, `tasks_ui`, `toolchain_selector` | Removed from init ✅ |
| Code Completion | `copilot`, `copilot_chat`, `copilot_ui`, `edit_prediction`, `edit_prediction_cli`, `edit_prediction_context`, `edit_prediction_metrics`, `edit_prediction_types`, `edit_prediction_ui` | Removed from init ✅ |
| Terminal & REPL | `terminal`, `terminal_view`, `repl` | Removed from init ✅ |
| Collaboration | `collab`, `collab_ui`, `call`, `channel`, `livekit_api`, `livekit_client`, `remote`, `remote_connection`, `remote_server` | Removed from init ✅ |
| Code Formatting | `prettier`, `snippet`, `snippet_provider`, `snippets_ui` | Removed from init ✅ |
| Vim | `vim`, `vim_mode_setting` | Removed from init ✅ |
| Auto-Update & Telemetry | `auto_update`, `auto_update_helper`, `auto_update_ui`, `telemetry`, `telemetry_events` | Removed from init ✅ |
| Misc Code-Specific | `diagnostics`, `node_runtime`, `dev_container`, `breadcrumbs`, `csv_preview`, `svg_preview` | Removed from init ✅ |
| LSP UI (panels only) | `language_tools`, `language_selector`, `language_onboarding` | Removed from init ✅ |

#### Keeping (but not using LSP features at runtime):

| Crate | Reason |
|-------|--------|
| `language` | Provides tree-sitter syntax highlighting, outline, auto-indent — needed for Markdown |
| `lsp` | Dependency of `language` — remove later in Phase 2 |
| `languages` | Strip to Markdown-only init (TODO) |
| `grammars` | Markdown grammar needed |

#### Explicitly Keeping:

| Category | Crates |
|----------|--------|
| GPUI + UI | `gpui`, `gpui_macos`, `gpui_platform`, `gpui_macros`, `ui`, `component`, `icons`, `theme`, `theme_settings`, `theme_selector` |
| Editor | `editor`, `text`, `rope`, `multi_buffer`, `sum_tree` |
| Workspace | `workspace`, `panel`, `sidebar`, `dock` (part of workspace) |
| Git | `git`, `git_ui`, `git_graph`, `git_hosting_providers` |
| Agent/AI | `agent`, `agent_ui`, `agent_settings`, `language_model`, `language_models`, `anthropic`, `open_ai`, `google_ai`, `ollama`, etc. |
| Markdown | `markdown`, `markdown_preview` |
| File Management | `project_panel`, `file_finder`, `worktree`, `fs` |
| Search | `search` |
| Outline | `outline`, `outline_panel` |
| Command Palette | `command_palette`, `command_palette_hooks` |
| Settings | `settings`, `settings_ui`, `settings_json` |
| Misc Keeping | `encoding_selector`, `line_ending_selector`, `journal`, `audio`, `extension`, `extension_host`, `extensions_ui` |

---

### Phase 1B: Identity & Cosmetics ✅ COMPLETE

**Goal:** It looks and feels like Aleph, not Zed.

- [x] Remove auto-authenticate on startup (no more keychain prompt for zed.dev)
- [x] Update app menus (code items removed, writer-friendly labels, says "Aleph")
- [x] Strip code-specific toolbar items from status bar
- [x] Rename "Zed Agent" → "Aleph" in agent panel, onboarding, conversation placeholder
- [x] Separate config directory (`~/.config/aleph/` instead of `~/.config/zed/`)
- [x] Separate data directory (`~/Library/Application Support/Aleph/` instead of Zed's)
- [x] Default panel layout: project/outline/git left, agent + threads sidebar right
- [x] Remove Zed cloud LLM provider (users bring their own API keys)
- [x] Remove AI onboarding upsell banner
- [x] Rebrand welcome page ("Welcome to Aleph", "Your agentic writing environment")
- [x] Add title_bar::init for proper titlebar rendering
- [x] Make title_bar work without `call` crate (ActiveCall now optional via try_global)

---

### Phase 2: Writer-Friendly Defaults ← CURRENT

**Goal:** The editor feels like a writing tool out of the box.

- [x] Default settings for writing: soft wrap, 18px font, comfortable line height, no line numbers, no scrollbar, no wrap guides
- [x] Defaults changed at source (`assets/settings/default.json`, `crates/paths`) — no workaround layers
- [x] Rewrite agent system prompt for writing collaborator persona
- [x] Rewrite agent templates (`create_file_prompt`, `diff_judge`) — no more "expert engineer"
- [x] Remove DiagnosticsTool and TerminalTool from agent tool set (15 writing-relevant tools remain)
- [x] Clean initial settings file (no dock position spam, proper template with `ensure_settings_file_exists`)
- [x] JSONC recognition for `~/.config/aleph/*.json`
- [x] Font size zoom keybindings (cmd+/cmd- handlers registered)
- [x] Welcome page shown on first open (instead of empty buffer)
- [x] User theme loading from `~/.config/aleph/themes/` (with hot-reload on file change)
- [x] Bundle "Aleph Latte" theme into `assets/themes/` as the built-in default
- [x] Set "Aleph Latte" as the default theme in `assets/settings/default.json`
- [x] Configurable system prompt: `~/.config/aleph/system_prompt.hbs` overrides embedded default (hot-reload on every request)
- [ ] Repurpose outline panel → chapter/section/scene navigator
- [ ] Markdown as the default/primary file type (new files open as .md)
- [ ] Distraction-free / focus mode (minimal chrome, centered text)
- [ ] Enable WebSearchTool with independent provider (Brave/SearXNG) — see `notes/zed-gated-features.md`
- [x] Enable UpdatePlanTool (remove feature flag gate)
- [ ] Clean up default settings file (remove code-specific settings, Zed references, irrelevant options)
- [x] New app icon (SF Pro ℵ on Aleph Latte `#f7f4e8`, pure black `#000000`)
- [x] Agent icon: replaced `zed_agent.svg` with geometric aleph letterform
- [ ] About dialog handler (currently no-ops)

### Backlog: Cleanup & Decoupling

**Goal:** Remove dead weight from the dependency tree. Not blocking, but nice to have.

- [ ] Strip `languages` init to Markdown-only
- [ ] Make `lsp` an optional dependency in `language` crate (decouple cleanly)
- [ ] Remove `task` dependency from `language` crate
- [ ] Remove `node_runtime` from `AppState` (transitive requirement currently)
- [x] Rename `IconName::ZedAgent` → `AlephAgent`, rename SVG file to `aleph_agent.svg`, update all references (~5 locations)

---

### Phase 3: Writing-Specific Features

**Goal:** Features that make Aleph *the* tool for writers.

- [ ] **Custom agent tools for fiction:**
  - Character database tool (agent can query/update character details)
  - Timeline tool (query/update story chronology)
  - World-building tool (locations, rules, lore)
  - Continuity checker (agent scans for contradictions)
- [ ] **Writing-focused agent prompts/personas:**
  - `~/.config/aleph/personas/` directory with plain-text persona files
  - Profile `persona` field references a file by name
  - System prompt template includes `{{{persona}}}` variable
  - Ship built-in personas: developmental editor, line editor, research assistant, brainstormer
  - UI: profile/persona switcher dropdown in agent panel
- [ ] **Word count & progress tracking:**
  - Session word count
  - Daily/weekly targets
  - Per-chapter/scene breakdown
  - Progress history
- [ ] **Export capabilities:**
  - Markdown → EPUB
  - Markdown → PDF (manuscript format)
  - Markdown → DOCX
- [ ] **Character bible panel** (sidebar panel tracking characters, relationships, arcs)
- [ ] **Timeline view** (visual chronology of your story)
- [ ] **Ambient writing mode** (audio crate — nature sounds, coffee shop, rain, etc.)
- [ ] **Read-aloud** (TTS reading your prose back)

---

## Development Notes

### Build Environment

- **Rust toolchain:** 1.94.1 (per `rust-toolchain.toml`)
- **Platform:** macOS (arm64)
- **Rustup path:** `/opt/homebrew/opt/rustup/bin`
- **Build command:** `export PATH="/opt/homebrew/opt/rustup/bin:$PATH" && cargo build -p aleph`
- **Check command:** `export PATH="/opt/homebrew/opt/rustup/bin:$PATH" && cargo check -p aleph`
- **Run command:** `./target/debug/aleph`
- **Prerequisites:** Xcode (for Metal shader compiler), cmake (for wasmtime/extensions)

### Key Files

- `crates/aleph/` — Main binary crate (our entry point)
- `crates/aleph/src/main.rs` — Application entry point
- `crates/aleph/src/aleph.rs` — App module (workspace init, menus, keymaps, panels)
- `crates/aleph/Cargo.toml` — Dependencies (stripped-down version of `crates/zed/Cargo.toml`)
- `assets/settings/default.json` — Default settings (modified at source for writer-friendly defaults)
- `crates/paths/src/paths.rs` — Config/data directory paths (changed to "aleph")
- `crates/agent/src/templates.rs` — System prompt template loading (supports config override)
- `~/.config/aleph/system_prompt.hbs` — User-overridable system prompt template (hot-reloaded)
- `ALEPH.md` — This file (master plan)

### Known Issues

- Default keymap has bindings for removed features (gracefully skipped, not user-visible)
- `languages::init` still loads all language grammars, not just Markdown (Phase 2 cleanup)
- `node_runtime` still in deps as transitive requirement of `AppState` struct (Phase 2 cleanup)
- Some deep UI text still references "Zed" (plan chips, URLs, etc.) — cosmetic, low priority
- JSONC trailing comma squiggles in settings file (file_types glob matches but highlighting still strict JSON in some contexts)
- WebSearchTool and UpdatePlanTool gated behind Zed cloud/feature flags — see `notes/zed-gated-features.md`

### Unresolved Questions

- **Window controls appear ~20% larger than Zed's** when running as bare binary. Likely a macOS quirk with unbundled executables (no `.app` bundle / Info.plist). Should resolve once we package as a proper app bundle. Not a code issue — the `rem_size`, `ui_font_size`, and `traffic_light_position` values are all identical to Zed's.

### Resumption Context

If picking up this work in a new session, read this file first. It contains the full plan, decisions made, and current progress. The app builds and runs — start from Phase 2 tasks.

---

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-29 | Name: "Aleph" | First letter, infinite cardinality, the beginning |
| 2026-04-29 | Keep Git | Commits as save points, diffs for revision tracking, branches for alternate storylines |
| 2026-04-29 | Keep `language` crate | Needed for tree-sitter Markdown highlighting and outline |
| 2026-04-29 | Keep encoding/line-ending selectors | Useful for international text and cross-platform manuscripts |
| 2026-04-29 | Keep journal | Daily writing notes — on brand |
| 2026-04-29 | Keep audio | Future ambient sounds / TTS |
| 2026-04-29 | Keep extension system | Future writing plugins |
| 2026-04-29 | Remove vim | No modal editing needed |
| 2026-04-29 | Remove terminal | Not a code execution environment |
| 2026-04-29 | Remove collab/calls | Too complex, tied to Zed servers |
| 2026-04-29 | Own the whole tree | Not a parallel build — modify any crate freely to remove Zed branding |
| 2026-04-29 | Change defaults at source | Don't add workaround layers in aleph crate; modify `assets/`, `crates/paths/`, etc. directly |
| 2026-04-29 | Use `load_asset_allow_partial_failure` for keymaps | Gracefully skip bindings for unregistered actions |
| 2026-04-29 | Base on stable release v0.233.10 | Known-good build baseline, branch: `aleph` |
| 2026-04-29 | Separate config dir `~/.config/aleph/` | Clean slate, no Zed settings interference |
| 2026-04-29 | Separate data dir `~/Library/Application Support/Aleph/` | No shared database/recent projects with Zed |
| 2026-04-29 | Make `ActiveCall` optional in title_bar | Use `try_global` instead of `global` — avoids requiring `call::init` |
| 2026-04-29 | Remove Zed cloud provider + Copilot provider | Users bring their own API keys; no Zed subscription dependency |
| 2026-04-29 | Rewrite agent system prompt for writing | Persona: developmental editor, line editor, brainstormer, researcher, continuity tracker |
| 2026-04-29 | Remove DiagnosticsTool + TerminalTool from agent | 15 tools remain, all writing-relevant |
| 2026-04-29 | Settings file created in main.rs | `ensure_settings_file_exists()` — not agent_ui's job |
| 2026-04-29 | Discovered Zed-gated features | WebSearchTool + UpdatePlanTool only work with Zed cloud. Notes in `notes/zed-gated-features.md` |
| 2026-04-29 | Configurable system prompt via `~/.config/aleph/system_prompt.hbs` | Hot-reload on every request; full Handlebars template override; future personas layer on top |
