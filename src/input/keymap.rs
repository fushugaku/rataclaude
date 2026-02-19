use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action::{Action, ActiveTab};
use crate::app::Focus;

pub fn map_key(key: KeyEvent, focus: Focus, active_tab: ActiveTab) -> Option<Action> {
    // Global bindings (always active, before anything else)
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Some(Action::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char(']')) => {
            let next = match active_tab {
                ActiveTab::ClaudeCode => ActiveTab::FileBrowser,
                ActiveTab::FileBrowser => ActiveTab::ClaudeCode,
            };
            return Some(Action::SwitchTab(next));
        }
        _ => {}
    }

    match active_tab {
        ActiveTab::ClaudeCode => map_claude_code_key(key, focus),
        ActiveTab::FileBrowser => map_file_browser_key(key),
    }
}

fn map_claude_code_key(key: KeyEvent, focus: Focus) -> Option<Action> {
    // Tab-level bindings for Claude Code tab
    match (key.modifiers, key.code) {
        (KeyModifiers::SHIFT, KeyCode::BackTab) => return Some(Action::ToggleFocus),
        (KeyModifiers::SHIFT, KeyCode::Tab) => return Some(Action::ToggleFocus),
        (KeyModifiers::NONE, KeyCode::BackTab) => return Some(Action::ToggleFocus),
        (KeyModifiers::CONTROL, KeyCode::Char('\\')) => return Some(Action::ResizePanes(0)),
        _ => {}
    }

    match focus {
        Focus::Pty => {
            // Forward everything to PTY
            Some(Action::PtyInput(key_to_bytes(key)))
        }
        Focus::GitStatus => map_git_status_key(key),
        Focus::DiffView => map_diff_view_key(key),
        Focus::PromptDialog => None, // handled directly in app
        Focus::FileBrowserLeft | Focus::FileBrowserRight => None,
    }
}

fn map_file_browser_key(key: KeyEvent) -> Option<Action> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
            Some(Action::FBNavDown)
        }
        (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
            Some(Action::FBNavUp)
        }
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Action::FBEnter),
        (KeyModifiers::NONE, KeyCode::Backspace) => Some(Action::FBParentDir),
        (KeyModifiers::NONE, KeyCode::Tab) => Some(Action::FBSwitchPanel),
        (KeyModifiers::NONE, KeyCode::PageUp) => Some(Action::FBPageUp),
        (KeyModifiers::NONE, KeyCode::PageDown) => Some(Action::FBPageDown),
        (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Action::FBCopy),
        (KeyModifiers::NONE, KeyCode::Char('m')) => Some(Action::FBMove),
        (KeyModifiers::NONE, KeyCode::Char('d')) => Some(Action::FBDelete),
        (KeyModifiers::NONE, KeyCode::Char('r')) => Some(Action::FBRename),
        (KeyModifiers::NONE, KeyCode::Char('n')) => Some(Action::FBMkdir),
        (KeyModifiers::NONE, KeyCode::Char('.')) => Some(Action::FBToggleHidden),
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => Some(Action::FBRefresh),
        _ => None,
    }
}

fn map_git_status_key(key: KeyEvent) -> Option<Action> {
    // Normalize: with kitty protocol, Shift+c may come as ('c', SHIFT) or ('C', SHIFT)
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
            Some(Action::GitNavDown)
        }
        (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
            Some(Action::GitNavUp)
        }
        (KeyModifiers::NONE, KeyCode::Char(' ')) => Some(Action::GitToggleStage),
        (KeyModifiers::NONE, KeyCode::Char('a')) => Some(Action::GitStageAll),
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Action::GitShowDiff),
        (KeyModifiers::NONE, KeyCode::Char('e')) => Some(Action::GitExpandFile),
        (KeyModifiers::NONE, KeyCode::Char('d')) => Some(Action::GitDiscardFile),
        (KeyModifiers::NONE, KeyCode::Char('s')) => Some(Action::SendToClaude),
        (KeyModifiers::SHIFT, KeyCode::Char('S')) | (KeyModifiers::SHIFT, KeyCode::Char('s')) => Some(Action::SendToClaudeWithPrompt),
        (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Action::Commit),
        (KeyModifiers::SHIFT, KeyCode::Char('C')) | (KeyModifiers::SHIFT, KeyCode::Char('c')) => Some(Action::CommitAndPush),
        (KeyModifiers::NONE, KeyCode::Char('p')) => Some(Action::Push),
        (KeyModifiers::SHIFT, KeyCode::Char('P')) | (KeyModifiers::SHIFT, KeyCode::Char('p')) => Some(Action::Pull),
        (KeyModifiers::NONE, KeyCode::Char('b')) => Some(Action::BranchList),
        (KeyModifiers::SHIFT, KeyCode::Char('B')) | (KeyModifiers::SHIFT, KeyCode::Char('b')) => Some(Action::CreateBranch),
        (KeyModifiers::NONE, KeyCode::Char('z')) => Some(Action::Stash),
        (KeyModifiers::SHIFT, KeyCode::Char('Z')) | (KeyModifiers::SHIFT, KeyCode::Char('z')) => Some(Action::StashPop),
        (KeyModifiers::NONE, KeyCode::Char('v')) => Some(Action::ToggleMultiSelect),
        _ => None,
    }
}

fn map_diff_view_key(key: KeyEvent) -> Option<Action> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
            Some(Action::DiffScrollDown)
        }
        (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
            Some(Action::DiffScrollUp)
        }
        (KeyModifiers::NONE, KeyCode::Char('h')) | (KeyModifiers::NONE, KeyCode::Left) => {
            Some(Action::DiffScrollLeft)
        }
        (KeyModifiers::NONE, KeyCode::Char('l')) | (KeyModifiers::NONE, KeyCode::Right) => {
            Some(Action::DiffScrollRight)
        }
        (KeyModifiers::SHIFT, KeyCode::Char('J')) | (KeyModifiers::SHIFT, KeyCode::Char('j')) => Some(Action::DiffNextHunk),
        (KeyModifiers::SHIFT, KeyCode::Char('K')) | (KeyModifiers::SHIFT, KeyCode::Char('k')) => Some(Action::DiffPrevHunk),
        (KeyModifiers::NONE, KeyCode::Char(' ')) => Some(Action::DiffToggleSelect),
        (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Char('q')) => {
            Some(Action::DiffClose)
        }
        (KeyModifiers::NONE, KeyCode::Char('s')) => Some(Action::DiffSendLines),
        (KeyModifiers::SHIFT, KeyCode::Char('S')) | (KeyModifiers::SHIFT, KeyCode::Char('s')) => Some(Action::SendToClaudeWithPrompt),
        _ => None,
    }
}

/// Convert a crossterm KeyEvent to the bytes that should be sent to a PTY
pub fn key_to_bytes(key: KeyEvent) -> Vec<u8> {
    let mut bytes = Vec::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char(c) => {
                let ctrl_byte = (c as u8).wrapping_sub(b'a').wrapping_add(1);
                bytes.push(ctrl_byte);
            }
            _ => {}
        }
        return bytes;
    }

    // Shift+Enter: send CSI u encoding so Claude Code sees it as newline (not submit)
    if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Enter {
        bytes.extend_from_slice(b"\x1b[13;2u");
        return bytes;
    }

    match key.code {
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(s.as_bytes());
        }
        KeyCode::Enter => bytes.push(b'\r'),
        KeyCode::Backspace => bytes.push(0x7f),
        KeyCode::Esc => bytes.push(0x1b),
        KeyCode::Tab => bytes.extend_from_slice(b"\x1b[9u"),
        KeyCode::Up => bytes.extend_from_slice(b"\x1b[A"),
        KeyCode::Down => bytes.extend_from_slice(b"\x1b[B"),
        KeyCode::Right => bytes.extend_from_slice(b"\x1b[C"),
        KeyCode::Left => bytes.extend_from_slice(b"\x1b[D"),
        KeyCode::Home => bytes.extend_from_slice(b"\x1b[H"),
        KeyCode::End => bytes.extend_from_slice(b"\x1b[F"),
        KeyCode::PageUp => bytes.extend_from_slice(b"\x1b[5~"),
        KeyCode::PageDown => bytes.extend_from_slice(b"\x1b[6~"),
        KeyCode::Delete => bytes.extend_from_slice(b"\x1b[3~"),
        KeyCode::Insert => bytes.extend_from_slice(b"\x1b[2~"),
        KeyCode::F(n) => {
            let seq = match n {
                1 => "\x1bOP",
                2 => "\x1bOQ",
                3 => "\x1bOR",
                4 => "\x1bOS",
                5 => "\x1b[15~",
                6 => "\x1b[17~",
                7 => "\x1b[18~",
                8 => "\x1b[19~",
                9 => "\x1b[20~",
                10 => "\x1b[21~",
                11 => "\x1b[23~",
                12 => "\x1b[24~",
                _ => "",
            };
            bytes.extend_from_slice(seq.as_bytes());
        }
        _ => {}
    }

    bytes
}
