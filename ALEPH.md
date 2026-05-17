# ℵ Aleph — Agentic Writing Environment

A fork of [Zed](https://zed.dev) re-imagined as a creative writing tool with first-class AI agent support.

Aleph takes Zed's blazing-fast editor, UI framework (GPUI), and AI agent infrastructure and strips away everything code-specific, replacing it with a purpose-built environment for fiction and non-fiction writers. Your AI assistant isn't autocomplete — it's a full collaborator that can read your manuscript, suggest prose improvements, catch continuity errors, research settings, and help you through the writing process.

## How it works

Aleph is always-local. Projects are directories of files on your machine (primarily Markdown). The AI agent executes tool calls locally and talks to LLM providers via API keys you configure yourself — no Zed cloud, no subscription, no hosted proxy.

The system is built in layers:

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

**GPUI** is the UI framework — a GPU-accelerated immediate-mode system that handles rendering, layout, input, and state management. Everything visible is built on it.

**The editor** (`editor`, `text`, `rope`, `multi_buffer`) is the core editing surface. The `text` crate uses a CRDT internally, which powers undo/redo. This is inherited from Zed's collaboration design — the CRDT could enable multi-user editing in the future, but for now it just gives us a solid undo system.

**Language support** (`language`, `grammars`, `lsp`) provides tree-sitter syntax highlighting, document outline, and auto-indent for Markdown. The `lsp` crate is present only as types — its `DiagnosticSeverity`, `LanguageServerId`, and `SymbolKind` types are used throughout `Buffer` and `language_core`. No LSP servers are ever spawned. No language intelligence runs. It's a type-level dependency with zero runtime cost.

**The agent** (`agent`, `agent_ui`, `acp_tools`, `acp_thread`, `language_model`, `language_models`) is the agentic writing core. It manages conversations, executes tools (file editing, search, git operations), and talks to LLM providers. The agent uses the terminal crate internally to run shell commands — the terminal isn't exposed as a user panel.

**The project** (`project`, `worktree`, `fs`) manages the file tree. It's been simplified to always-local mode: no remote constructors, no SSH connections, no collaboration state. A project is just a set of local worktrees.

### Where the seams show

The codebase still carries some Zed DNA that doesn't serve Aleph but is entangled enough to require focused refactoring to remove:

- **Telemetry.** ~20 crates call `telemetry::event!(...)` functions that do nothing (no server to report to). Pure dead code, but spread across many files. Removing it is mechanical but tedious.
- **Prettier.** The project crate has a formatting subsystem (`prettier_store`) that manages Node.js Prettier instances. Never invoked for Markdown, but woven into `lsp_store.rs` at ~13 points.
- **Task runner.** Zed's "run build/test" system. The `language` crate depends on it for `RunnableTag` and `ShellKind` types. The fix is extracting those types and deleting the rest.
- **Snippets.** Code snippet expansion in the editor. Could be repurposed for writing templates or just removed.
- **Ghost text predictions.** The `edit_prediction_types` trait system for Copilot-style completions. Deep in editor. Could theoretically be repurposed for prose suggestions.

None of these run at runtime — they're compile-time baggage that makes the build slower and the code noisier than it needs to be.

## Where it's going

The next layer of work is making Aleph feel like a *writing tool*, not just "a code editor that doesn't have code features":

- **Markdown as the default.** New files open as `.md`. The file system revolves around prose.
- **Chapter/scene navigation.** The outline panel repurposed to show document structure (headings as chapters, scenes, sections) rather than code symbols.
- **Distraction-free mode.** Minimal chrome, centered text, just you and the words.
- **Word count and progress.** Session counts, daily targets, per-chapter breakdowns.
- **Writing agent tools.** Character database, timeline, world-building, continuity checking — tools the agent can call to maintain story state.
- **Agent personas.** Switchable profiles: developmental editor, line editor, brainstormer, researcher.
- **Export.** Markdown → EPUB, PDF (manuscript format), DOCX.
- **Ambient audio.** Simple playback (rain, coffee shop) using a lightweight library — not the WebRTC voice infrastructure we removed.

Longer term, the open question is whether Aleph should go WYSIWYG — rendering Markdown visually (styled headers, inline images, rendered bold/italic) rather than showing source. That would be a major editor surface overhaul but would strongly differentiate it from "a code editor for prose."

## Building and running

- **Rust toolchain:** 1.94.1 (per `rust-toolchain.toml`)
- **Platform:** macOS (arm64)
- **Build:** `export PATH="/opt/homebrew/opt/rustup/bin:$PATH" && cargo build -p aleph`
- **Check:** `export PATH="/opt/homebrew/opt/rustup/bin:$PATH" && cargo check -p aleph`
- **Run:** `./target/debug/aleph`
- **Prerequisites:** Xcode (Metal shader compiler), cmake (wasmtime/extensions)

Key files:

| Path | What it does |
|------|-------------|
| `crates/aleph/src/main.rs` | Startup — initializes subsystems, creates the window, loads settings |
| `crates/aleph/src/aleph.rs` | Registers panels, menus, keymaps, workspace behavior |
| `crates/aleph/Cargo.toml` | What the binary depends on directly |
| `assets/settings/default.json` | Default settings (soft wrap, no line numbers, comfortable spacing) |
| `crates/paths/src/paths.rs` | Config and data directory locations |
| `crates/agent/src/templates.rs` | System prompt template loading — supports `~/.config/aleph/system_prompt.hbs` override with hot-reload |

## Decisions

Settled architectural choices. If the reasoning still holds, the decision stands.

- **The name.** Aleph (ℵ) — the first letter, infinite cardinality, the beginning.
- **Always-local.** No remote editing, no SSH, no collaboration. Removes an entire class of connection-state complexity.
- **Own the whole tree.** We modify any crate freely. Full transformation, not coexistence with Zed.
- **Change defaults at source.** Modify `assets/`, `crates/paths/`, `crates/editor/` directly — no workaround layers.
- **Users bring their own API keys.** No Zed cloud dependency. Configure Anthropic/OpenAI/Ollama/etc. directly.
- **Keep Git.** Commits as save points, diffs for revision tracking, branches for alternate storylines.
- **Keep the extension system.** Future writing plugins will use it.
- **Don't fight `lsp` or the CRDT.** Both are type-level dependencies with zero runtime cost. Not worth the surgery.
- **Keep terminal for the agent.** Shell execution is needed internally. Not user-facing.
- **Separate config and data.** `~/.config/aleph/` for settings, `~/Library/Application Support/Aleph/` for databases. Clean separation from Zed.
- **Keep encoding/line-ending selectors.** Useful for international text and cross-platform manuscripts. Status bar shows LF/CRLF at a glance.
- **Configurable system prompt.** `~/.config/aleph/system_prompt.hbs` with hot-reload. Full Handlebars templating.
