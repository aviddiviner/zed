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

### Phase 1A: Fork & Strip ← CURRENT

**Goal:** A building, running app with code-specific features removed.

#### Steps:

- [ ] Create `crates/aleph` binary crate (fork of `crates/zed`)
- [ ] Strip dependencies from `aleph/Cargo.toml`:
  - [x] Identify removals (see Removal List below)
- [ ] Strip initialization code in `main.rs` — don't init removed features
- [ ] Strip `languages` init down to Markdown only
- [ ] Confirm it builds and opens a window with: editor + file tree + agent panel
- [ ] Update workspace Cargo.toml to include `crates/aleph`

#### Removal List (confirmed by human review):

| Group | Crates | Status |
|-------|--------|--------|
| Debugger & Tasks | `dap`, `dap_adapters`, `debugger_ui`, `debugger_tools`, `debug_adapter_extension`, `task`, `tasks_ui`, `toolchain_selector` | Approved ✅ |
| Code Completion | `copilot`, `copilot_chat`, `copilot_ui`, `edit_prediction`, `edit_prediction_cli`, `edit_prediction_context`, `edit_prediction_metrics`, `edit_prediction_types`, `edit_prediction_ui` | Approved ✅ |
| Terminal & REPL | `terminal`, `terminal_view`, `repl` | Approved ✅ |
| Collaboration | `collab`, `collab_ui`, `call`, `channel`, `livekit_api`, `livekit_client`, `remote`, `remote_connection`, `remote_server` | Approved ✅ |
| Code Formatting | `prettier`, `snippet`, `snippet_provider`, `snippets_ui` | Approved ✅ |
| Vim | `vim`, `vim_mode_setting` | Approved ✅ |
| Auto-Update & Telemetry | `auto_update`, `auto_update_helper`, `auto_update_ui`, `telemetry`, `telemetry_events` | Approved ✅ |
| Misc Code-Specific | `diagnostics`, `node_runtime`, `dev_container`, `breadcrumbs`, `csv_preview`, `svg_preview` | Approved ✅ |
| LSP UI (panels only) | `language_tools`, `language_selector`, `language_onboarding` | Approved ✅ |

#### Keeping (but not using LSP features at runtime):

| Crate | Reason |
|-------|--------|
| `language` | Provides tree-sitter syntax highlighting, outline, auto-indent — needed for Markdown |
| `lsp` | Dependency of `language` — remove later in Phase 2 |
| `languages` | Strip to Markdown-only init |
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

### Phase 1B: Identity & Cosmetics

**Goal:** It looks and feels like Aleph, not Zed.

- [ ] Rename app to "Aleph" in bundle metadata
- [ ] Update app menus (remove code-specific menu items)
- [ ] New app icon (placeholder fine initially)
- [ ] Strip code-specific toolbar items (LSP status, diagnostics count, etc.)
- [ ] Default panel layout: file tree left, agent right, editor center

---

### Phase 2: Writer-Friendly Defaults

**Goal:** The editor feels like a writing tool out of the box.

- [ ] Default settings for writing: word wrap on, larger font size, serif font option
- [ ] Repurpose outline panel → chapter/section/scene navigator
- [ ] Markdown as the default/primary file type
- [ ] Agent panel front-and-center in default layout
- [ ] Make `lsp` an optional dependency in `language` crate (decouple cleanly)
- [ ] Remove `task` dependency from `language` crate
- [ ] Distraction-free / focus mode (minimal chrome, centered text)

---

### Phase 3: Writing-Specific Features

**Goal:** Features that make Aleph *the* tool for writers.

- [ ] **Custom agent tools for fiction:**
  - Character database tool (agent can query/update character details)
  - Timeline tool (query/update story chronology)
  - World-building tool (locations, rules, lore)
  - Continuity checker (agent scans for contradictions)
- [ ] **Writing-focused agent prompts/personas:**
  - Developmental editor persona
  - Line editor persona
  - Research assistant persona
  - Brainstorming partner persona
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

### Key Files

- `crates/aleph/` — Main binary crate (our entry point)
- `crates/aleph/src/main.rs` — Application entry point
- `crates/aleph/Cargo.toml` — Dependencies (stripped-down version of `crates/zed/Cargo.toml`)
- `ALEPH.md` — This file (master plan)

### Resumption Context

If picking up this work in a new session, read this file first. It contains the full plan, decisions made, and current progress. Check the Phase 1A checkboxes to see what's been completed.

---

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-01-XX | Name: "Aleph" | First letter, infinite cardinality, the beginning |
| 2025-01-XX | Keep Git | Commits as save points, diffs for revision tracking, branches for alternate storylines |
| 2025-01-XX | Keep `language` crate | Needed for tree-sitter Markdown highlighting and outline |
| 2025-01-XX | Keep encoding/line-ending selectors | Useful for international text and cross-platform manuscripts |
| 2025-01-XX | Keep journal | Daily writing notes — on brand |
| 2025-01-XX | Keep audio | Future ambient sounds / TTS |
| 2025-01-XX | Keep extension system | Future writing plugins |
| 2025-01-XX | Remove vim | No modal editing needed |
| 2025-01-XX | Remove terminal | Not a code execution environment |
| 2025-01-XX | Remove collab/calls | Too complex, tied to Zed servers |
| 2025-01-XX | Strategy: new binary, don't delete workspace crates | Avoids cascading Cargo dependency hell |
