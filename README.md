# rataclaude

A split-pane TUI that runs [Claude Code](https://docs.anthropic.com/en/docs/claude-code) in a PTY on the left and provides lazygit-style git management on the right. Select files or diffs on the right and inject them as `@file` references directly into Claude's input.

![Rust](https://img.shields.io/badge/rust-stable-orange) ![macOS](https://img.shields.io/badge/platform-macOS-blue)

## Features

- **PTY integration** — Claude Code runs in a real pseudo-terminal with full color, cursor, and resize support
- **Git status** — view modified, staged, and untracked files with status icons
- **Diff preview** — syntax-highlighted, scrollable unified diff with line numbers and hunk headers
- **Send to Claude** — select files and send them as `@file` references with an optional prompt
- **Git operations** — stage, unstage, commit, push, pull, stash, branch create/checkout, all from the TUI
- **Mouse support** — click to switch panes, scroll diffs
- **Syntax highlighting** — 70+ languages via syntect, including Swift via custom syntax definition

## Requirements

- macOS (arm64 or x86_64)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) CLI (`claude`) in PATH
- Rust toolchain (to build from source)

## Install

```sh
cargo install --path .
```

## Usage

Run inside a git repository:

```sh
rataclaude
```

## Key Bindings

### Global

| Key | Action |
|-----|--------|
| `Tab` | Toggle focus between PTY and Git panes |
| `Ctrl+q` | Quit |

### Git Status (right pane)

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate files |
| `Space` | Stage / unstage file |
| `a` | Stage all |
| `Enter` | Show diff |
| `s` | Send selected files to Claude |
| `S` | Send with prompt |
| `v` | Toggle multi-select |
| `c` | Commit |
| `C` | Commit and push |
| `p` | Push |
| `P` | Pull |
| `b` | List branches |
| `B` | Create branch |
| `z` | Stash |
| `Z` | Stash pop |
| `d` | Discard changes |

### Diff View

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll |
| `Esc` | Back to status list |

## Architecture

Three async tasks feed a single `mpsc` channel:

1. **PTY reader** — reads Claude Code output, sends `PtyOutput` events
2. **Crossterm EventStream** — keyboard, mouse, resize events
3. **Tick timer** — periodic git state refresh

The main loop receives events, updates state, and redraws via ratatui. Terminal emulation is handled by `vt100`, which maintains a screen buffer mapped cell-by-cell to ratatui's buffer.

## License

MIT
