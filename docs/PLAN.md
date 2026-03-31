# recall — Rust TUI Note App

## Overview

CLI tool for quick note-taking with a scrollable TUI browser. Notes stored in a single Obsidian-compatible markdown file.

## CLI Interface

```
r                     → open TUI to browse notes
r "some note"         → add a note (shorthand)
r add "some note"     → add a note (explicit)
```

## Storage

- Location: `~/.recall/notes.md`
- Format (Obsidian-compatible checkbox):

```markdown
- [ ] 2026-03-28 14:30:00
this is something i want to remember

- [ ] 2026-03-28 15:00:00
another note here
```

## Dependencies (Cargo.toml)

| Crate | Purpose |
|---|---|
| `clap` (derive) | CLI argument parsing |
| `ratatui` + `crossterm` | TUI rendering |
| `chrono` | Timestamps |
| `dirs` | Resolve `~/.recall/` path |

## Project Structure

```
src/
├── main.rs       # CLI entry (clap), dispatches to add or TUI
├── note.rs       # Note struct, parse/format
├── storage.rs    # Read/write ~/.recall/notes.md
└── tui.rs        # ratatui scrollable list view
```

## TUI Behavior

- Full-screen scrollable list (newest first)
- Each row shows timestamp + note preview (truncated to terminal width)
- Navigation: arrow keys / `j`/`k` to scroll
- Quit: `q` / `Esc`
- Status bar at bottom with keybindings hint

## Implementation Steps

1. `cargo init` + configure `Cargo.toml` with dependencies
2. `note.rs` — `Note` struct with `timestamp` (DateTime) and `text` (String), serialization to checkbox format
3. `storage.rs` — ensure `~/.recall/notes.md` exists, parse all notes, append new note
4. `main.rs` — clap app: positional arg for note text (optional). If present → append note and exit. If absent → launch TUI
5. `tui.rs` — ratatui app with `List` widget, scrollable, keyboard-driven
6. Wire together and test
7. Add `alias r=recall` to shell profile

## Deferred

- **fzf integration** — optional search mode (`r find` / `r search`), gated behind runtime check for fzf availability
