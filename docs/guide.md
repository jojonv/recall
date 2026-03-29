# recall — User Guide

A CLI note-taking tool with a fullscreen TUI browser and configurable notebooks. Notes are stored in a single Obsidian-compatible markdown file.

## Setup

Build and install:

```
cargo build --release
```

Add an alias to your shell profile for quick access:

```
alias r=recall
```

All examples below use `r` as the command.

## Quick Reference

| Command | Action |
|---|---|
| `r` | Open TUI browser |
| `r "buy groceries"` | Add note (shorthand) |
| `r buy groceries` | Add note (unquoted) |
| `r w` | Open TUI for `w` notebook |
| `r w "meeting at 3"` | Add note to `w` notebook (shorthand) |
| `r w meeting at 3` | Add note to `w` notebook (unquoted) |

## Adding Notes

### Default notebook

Add a note to the default notebook:

```
r "deploy v2 before lunch"
r deploy v2 before lunch
```

The first is a quoted positional argument. The second is unquoted — any unrecognized input is treated as note text.

### Specific notebook

If you have notebooks configured (see [Notebooks](#notebooks)):

```
r w "standup: blocked on API"
r w standup: blocked on API
```

`w` is detected as a notebook name and the rest becomes note text.

### Multiline notes

Notes can contain newlines. How they appear depends on your shell:

```
r "line one
line two"
```

Or using `$'...'` syntax (bash/zsh):

```
r $'header:\n- item one\n- item two'
```

### Validation

Empty and whitespace-only notes are rejected with an error.

## TUI Browser

Running `r` with no arguments opens an interactive fullscreen TUI:

```
r
```

### Display

- Notes are shown newest-first in a scrollable list
- Each row displays the timestamp and the first line of the note text
- The selected item is highlighted with a blue background and `>>` prefix
- Completed notes appear dimmed with strikethrough

### Keyboard shortcuts

| Key | Action |
|---|---|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `d` | Toggle done/undone on selected note |
| `q` / `Esc` | Quit |

Navigation wraps around — pressing `j` on the last item selects the first, and vice versa.

The done toggle (`d`) persists immediately to disk. Toggling a note back to undone also persists.

### Notebook TUI

To browse a specific notebook in the TUI:

```
r w
```

This opens the TUI showing only notes from the `w` notebook.

## Notebooks

Notebooks let you organize notes into separate files, each with a short alias for quick access from the CLI.

### Configuration

Edit `~/.recall/config.toml`:

```toml
file = "~/.recall/notes.md"

[notebooks]
w = "~/notes/work.md"
p = "~/notes/personal.md"
j = "~/journal/daily.md"
```

- `file` — path to the default notebook (optional, defaults to `~/.recall/notes.md`)
- `[notebooks]` — map of alias to file path (optional)
- Paths support `~/` expansion

### Notebook aliases

Aliases are the keys in the `[notebooks]` section. Choose short, memorable names:

```toml
[notebooks]
w = "~/notes/work.md"
p = "~/notes/personal.md"
```

### How notebook routing works

When you run a command with arguments that aren't recognized subcommands:

1. The first argument is checked against your notebook aliases
2. If it matches a notebook — the command targets that notebook's file
3. If it doesn't match — all arguments are treated as note text for the default notebook

```
r w "meeting notes"       → "w" is a notebook → add to work.md
r "random thought"        → not a notebook    → add to default notes.md
r random thought here     → not a notebook    → add to default (joined as one note)
```

### Notebook commands

Once a notebook is matched, the remaining arguments determine the action:

| Input | Action |
|---|---|
| `r w` | Open TUI for `w` |
| `r w "some text"` | Add note to `w` (shorthand) |
| `r w some text` | Add note to `w` (unquoted) |

## Storage Format

Notes are stored in Obsidian-compatible markdown checkbox format:

```markdown
- [ ] 2026-03-29 14:30:00
deploy v2 before lunch

- [x] 2026-03-29 09:00:00
fix bug #42

- [ ] 2026-03-28 17:15:00
call dentist
```

Each note consists of:

- A header line: `- [ ] YYYY-MM-DD HH:MM:SS` (open) or `- [x] YYYY-MM-DD HH:MM:SS` (done)
- One or more lines of note text
- A blank line separating notes

Because the format is plain markdown, you can edit your note files directly in any text editor or use them in Obsidian.

### File locations

| File | Default path | Purpose |
|---|---|---|
| Notes | `~/.recall/notes.md` | Default notebook |
| Config | `~/.recall/config.toml` | Configuration (optional) |

Both paths can be overridden in the config file. Parent directories are created automatically when needed.

## Configuration

The optional config file at `~/.recall/config.toml`:

```toml
# Override default notes file location
file = "~/vault/notes.md"

# Define notebook aliases
[notebooks]
w = "~/notes/work.md"
p = "~/notes/personal.md"
```

All fields are optional. If the config file doesn't exist, recall uses defaults.

### Config reference

| Field | Type | Default | Description |
|---|---|---|---|
| `file` | string | `~/.recall/notes.md` | Path to default notebook |
| `notebooks` | map | _(none)_ | Alias-to-path mappings |

Paths in config support `~/` prefix for home directory expansion.
