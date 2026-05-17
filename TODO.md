# TODO

## Cleanup

- [ ] Remove `telemetry` + `telemetry_events` (~20 crates, mechanical grep-and-delete)
- [ ] Remove `prettier` (prettier_store.rs ~1000 lines + 13 refs in lsp_store.rs)
- [ ] Remove `task` crate (extract ShellKind to language_core, stub ContextProvider/RunnableTag)
- [ ] Remove `snippet` + `snippet_provider` (or repurpose for writing templates)
- [ ] Remove `edit_prediction_types` (or repurpose for prose ghost-text)
- [ ] Strip grammars to writing languages only (md, json, yaml, diff, gitcommit)
- [ ] Remove `node_runtime` from AppState (transitive requirement through project, dead code)
- [ ] Remove `UserStore` from AppState/Project (transitive, zero runtime cost but dead code)
- [ ] Clean up default settings file (remove code-specific settings, Zed references, irrelevant options)

## Features

- [ ] Markdown as default file type (new files → .md, new buffers default to Markdown syntax)
- [ ] Chapter/scene navigator (outline panel + tree-sitter Markdown headings)
- [ ] Distraction-free / focus mode (minimal chrome, centered text column, hidden panels)
- [ ] Word count and progress tracking (session count, daily/weekly targets, per-chapter breakdown)
- [ ] Writing agent personas (`~/.config/aleph/personas/`, profile picker in agent panel)
- [ ] Export (Markdown → EPUB, PDF manuscript format, DOCX)
- [ ] Character bible panel (sidebar tracking characters, relationships, arcs)
- [ ] Timeline view (visual chronology of your story)
- [ ] First-run onboarding screen (base keymap picker, Aleph-appropriate version of Zed's onboarding)
- [ ] About dialog (currently no-ops — should show Aleph version, Zed base version, git commit)

## Bugs

- [ ] **MCP servers have no shell environment.**
      Servers launched from the Dock get a minimal macOS GUI environment (no PATH, no homebrew, etc.). Root cause: `ProjectEnvironment::default_environment()` isn't available at server startup time — the `ContextServerStore.project` weak ref is likely None when `maintain_servers` fires during init. Workaround: explicit `"env": {"PATH": "..."}` in each server's config. Likely fix: use `shell_env::capture()` (which already handles fish, noisy output, etc.) to resolve the user's shell environment before servers start, then inject via `command.env`.

- [ ] **WebSearchTool gated to Zed cloud.**
      The `supports_provider` check only returns true for the Zed cloud LLM provider. Since Aleph uses bring-your-own-key providers, web search is disabled. See `notes/zed-gated-features.md`. Fix: bypass the provider check or implement an independent backend (Brave API, SearXNG).

- [ ] **Window controls are 14pt on macOS Tahoe.**
      SDK-linked behavior — macOS reads the `LC_BUILD_VERSION` sdk field from the binary and renders larger traffic-light buttons for SDK 26+ apps. Not a code bug. Options: embrace the new size (adjust `TRAFFIC_LIGHT_PADDING` in `crates/ui/src/utils/constants.rs`) or pin to older SDK via `-Wl,-platform_version,macos,10.15.7,15.0` linker flag. See `crates/title_bar/build.rs` for the `macos_sdk_26` cfg.

- [ ] **Default keymap has bindings for removed features.**
      Gracefully skipped (not user-visible), but noisy in the binding resolution.

- [ ] **Some UI text still references "Zed".**
      Plan chips, URLs, possibly other deep spots.

- [ ] **JSONC trailing comma squiggles.**
      file_types glob matches `.json` config files to JSONC, but highlighting uses strict JSON grammar in some paths — so valid trailing commas get squiggles.
