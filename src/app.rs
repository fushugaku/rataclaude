use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::action::{Action, FocusTarget};
use crate::event::AppEvent;
use crate::git::diff::FileDiff;
use crate::git::operations::GitOps;
use crate::git::repo::GitRepo;
use crate::git::status::FileStatus;
use crate::input::handler;
use crate::pty::manager::PtyManager;
use crate::pty::terminal_emulator::TerminalEmulator;
use crate::ui::diff_view::DiffViewState;
use crate::ui::layout::AppLayout;
use crate::ui::prompt_dialog::{PromptDialogState, PromptMode};
use crate::ui::status_list::StatusListState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Pty,
    GitStatus,
    DiffView,
    PromptDialog,
}

pub struct App {
    pub running: bool,
    pub focus: Focus,
    pub layout: AppLayout,
    pub emulator: TerminalEmulator,
    pub pty: PtyManager,
    pub git_repo: Option<GitRepo>,
    pub git_ops: Option<GitOps>,
    pub files: Vec<FileStatus>,
    pub current_diff: Option<FileDiff>,
    pub branch: String,
    pub status_state: StatusListState,
    pub diff_state: DiffViewState,
    pub prompt_state: PromptDialogState,
    pub last_pty_area: Rect,
    pub error_message: Option<String>,
    // Stored pane rects for mouse hit-testing (set during draw)
    pub pty_rect: Rect,
    pub git_status_rect: Rect,
    pub diff_rect: Rect,
}

impl App {
    pub fn new(pty: PtyManager, cols: u16, rows: u16) -> Self {
        let workdir = std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let git_repo = GitRepo::open(&workdir).ok();
        let git_ops = git_repo.as_ref()
            .and_then(|r| r.workdir())
            .map(|p| GitOps::new(&p.to_string_lossy()));

        let branch = git_repo.as_ref()
            .and_then(|r| r.branch_name().ok())
            .unwrap_or_else(|| "N/A".to_string());

        Self {
            running: true,
            focus: Focus::Pty,
            layout: AppLayout::new(),
            emulator: TerminalEmulator::new(rows, cols),
            pty,
            git_repo,
            git_ops,
            files: Vec::new(),
            current_diff: None,
            branch,
            status_state: StatusListState::new(),
            diff_state: DiffViewState::new(),
            prompt_state: PromptDialogState::new(),
            last_pty_area: Rect::default(),
            error_message: None,
            pty_rect: Rect::default(),
            git_status_rect: Rect::default(),
            diff_rect: Rect::default(),
        }
    }

    pub fn refresh_git(&mut self) {
        if let Some(ref repo) = self.git_repo {
            match repo.status_list() {
                Ok(files) => self.files = files,
                Err(e) => self.error_message = Some(format!("Git status error: {}", e)),
            }
            if let Ok(branch) = repo.branch_name() {
                self.branch = branch;
            }
        }
    }

    pub fn refresh_diff(&mut self) {
        if let Some(ref repo) = self.git_repo {
            if let Some(idx) = self.status_state.selected_index() {
                if let Some(file) = self.files.get(idx) {
                    let staged = file.stage_state == crate::git::status::StageState::Staged;
                    match repo.diff_file(&file.path, staged) {
                        Ok(diff) => {
                            self.diff_state.set_file(&file.path);
                            self.current_diff = Some(diff);
                        }
                        Err(_) => {
                            self.current_diff = None;
                        }
                    }
                }
            }
        }
    }

    /// Store pane rects during draw for mouse hit-testing
    pub fn update_rects(&mut self, pty: Rect, git_status: Rect, diff: Rect) {
        self.pty_rect = pty;
        self.git_status_rect = git_status;
        self.diff_rect = diff;
    }

    fn hit_test(&self, col: u16, row: u16) -> Option<FocusTarget> {
        if rect_contains(self.pty_rect, col, row) {
            Some(FocusTarget::Pty)
        } else if rect_contains(self.git_status_rect, col, row) {
            Some(FocusTarget::GitStatus)
        } else if rect_contains(self.diff_rect, col, row) {
            Some(FocusTarget::DiffView)
        } else {
            None
        }
    }

    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(key) => {
                // Handle prompt dialog input directly
                if self.focus == Focus::PromptDialog {
                    self.handle_prompt_key(key).await?;
                    return Ok(());
                }

                if let Some(action) = handler::handle_key(key, self.focus) {
                    self.handle_action(action).await?;
                }
            }
            AppEvent::PtyOutput(data) => {
                self.emulator.process(&data);
            }
            AppEvent::PtyExited => {
                self.running = false;
            }
            AppEvent::Resize(_, _) => {}
            AppEvent::Tick | AppEvent::GitRefresh => {
                self.refresh_git();
            }
            AppEvent::Mouse(mouse) => {
                self.handle_mouse(mouse).await?;
            }
        }
        Ok(())
    }

    async fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Don't switch focus if prompt is open
                if self.focus == Focus::PromptDialog {
                    return Ok(());
                }
                if let Some(target) = self.hit_test(mouse.column, mouse.row) {
                    self.handle_action(Action::FocusPane(target)).await?;
                }
            }
            MouseEventKind::ScrollDown => {
                if rect_contains(self.diff_rect, mouse.column, mouse.row) {
                    self.handle_action(Action::DiffScrollAmount(3)).await?;
                } else if self.focus == Focus::Pty
                    && rect_contains(self.pty_rect, mouse.column, mouse.row)
                {
                    // Forward scroll to PTY as arrow keys
                    self.pty.write_input(b"\x1b[B").await?;
                    self.pty.write_input(b"\x1b[B").await?;
                    self.pty.write_input(b"\x1b[B").await?;
                }
            }
            MouseEventKind::ScrollUp => {
                if rect_contains(self.diff_rect, mouse.column, mouse.row) {
                    self.handle_action(Action::DiffScrollAmount(-3)).await?;
                } else if self.focus == Focus::Pty
                    && rect_contains(self.pty_rect, mouse.column, mouse.row)
                {
                    self.pty.write_input(b"\x1b[A").await?;
                    self.pty.write_input(b"\x1b[A").await?;
                    self.pty.write_input(b"\x1b[A").await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.running = false;
            }
            Action::ToggleFocus => {
                self.focus = match self.focus {
                    Focus::Pty => Focus::GitStatus,
                    Focus::GitStatus | Focus::DiffView => Focus::Pty,
                    Focus::PromptDialog => Focus::PromptDialog,
                };
            }
            Action::FocusPane(target) => {
                self.focus = match target {
                    FocusTarget::Pty => Focus::Pty,
                    FocusTarget::GitStatus => Focus::GitStatus,
                    FocusTarget::DiffView => {
                        if self.current_diff.is_some() {
                            Focus::DiffView
                        } else {
                            Focus::GitStatus
                        }
                    }
                };
            }
            Action::ResizePanes(_) => {
                let current = self.layout.split_percent;
                self.layout.split_percent = match current {
                    0..=45 => 60,
                    46..=65 => 80,
                    _ => 40,
                };
            }
            Action::PtyInput(bytes) => {
                self.pty.write_input(&bytes).await?;
            }
            Action::GitNavUp => {
                let len = self.files.len();
                self.status_state.move_up(len);
                self.refresh_diff();
            }
            Action::GitNavDown => {
                let len = self.files.len();
                self.status_state.move_down(len);
                self.refresh_diff();
            }
            Action::GitToggleStage => {
                if let Some(idx) = self.status_state.selected_index() {
                    if let Some(file) = self.files.get(idx) {
                        if let Some(ref ops) = self.git_ops {
                            let path = file.path.clone();
                            let result = if file.stage_state == crate::git::status::StageState::Staged {
                                ops.unstage_file(&path)
                            } else {
                                ops.stage_file(&path)
                            };
                            if let Err(e) = result {
                                self.error_message = Some(format!("{}", e));
                            }
                            self.refresh_git();
                            self.refresh_diff();
                        }
                    }
                }
            }
            Action::GitStageAll => {
                if let Some(ref ops) = self.git_ops {
                    if let Err(e) = ops.stage_all() {
                        self.error_message = Some(format!("{}", e));
                    }
                    self.refresh_git();
                }
            }
            Action::GitShowDiff => {
                self.refresh_diff();
                if self.current_diff.is_some() {
                    self.focus = Focus::DiffView;
                }
            }
            Action::GitDiscardFile => {
                if let Some(idx) = self.status_state.selected_index() {
                    if let Some(file) = self.files.get(idx) {
                        if let Some(ref ops) = self.git_ops {
                            let path = file.path.clone();
                            if let Err(e) = ops.discard_file(&path) {
                                self.error_message = Some(format!("{}", e));
                            }
                            self.refresh_git();
                        }
                    }
                }
            }
            Action::DiffScrollUp => {
                self.diff_state.scroll_up(1);
            }
            Action::DiffScrollDown => {
                let max = self.current_diff.as_ref()
                    .map(|d| d.total_lines() as u16)
                    .unwrap_or(0);
                self.diff_state.scroll_down(1, max);
            }
            Action::DiffScrollAmount(delta) => {
                if delta < 0 {
                    self.diff_state.scroll_up((-delta) as u16);
                } else {
                    let max = self.current_diff.as_ref()
                        .map(|d| d.total_lines() as u16)
                        .unwrap_or(0);
                    self.diff_state.scroll_down(delta as u16, max);
                }
            }
            Action::DiffNextHunk => {
                if let Some(ref diff) = self.current_diff {
                    let lines = diff.all_lines();
                    let current = self.diff_state.scroll as usize;
                    for (i, line) in lines.iter().enumerate().skip(current + 1) {
                        if line.kind == crate::git::diff::DiffLineKind::HunkHeader {
                            self.diff_state.scroll = i as u16;
                            break;
                        }
                    }
                }
            }
            Action::DiffPrevHunk => {
                if let Some(ref diff) = self.current_diff {
                    let lines = diff.all_lines();
                    let current = self.diff_state.scroll as usize;
                    for i in (0..current).rev() {
                        if lines[i].kind == crate::git::diff::DiffLineKind::HunkHeader {
                            self.diff_state.scroll = i as u16;
                            break;
                        }
                    }
                }
            }
            Action::DiffClose => {
                self.focus = Focus::GitStatus;
            }
            Action::SendToClaude => {
                let selected = self.status_state.selected_files(&self.files);
                if !selected.is_empty() {
                    let file_refs: Vec<String> = selected.iter()
                        .map(|f| format!("@{}", f.path))
                        .collect();
                    let cmd = format!("{}\n", file_refs.join(" "));
                    self.pty.inject_input(&cmd).await?;
                    self.focus = Focus::Pty;
                }
            }
            Action::SendToClaudeWithPrompt => {
                let selected = self.status_state.selected_files(&self.files);
                if !selected.is_empty() {
                    let files: Vec<String> = selected.iter()
                        .map(|f| f.path.clone())
                        .collect();
                    self.prompt_state.open_send(files);
                    self.focus = Focus::PromptDialog;
                }
            }
            Action::ToggleMultiSelect => {
                self.status_state.toggle_multi_select();
                if self.status_state.multi_select {
                    self.status_state.toggle_select();
                }
            }
            Action::Commit => {
                self.prompt_state.open_commit();
                self.focus = Focus::PromptDialog;
            }
            Action::CommitAndPush => {
                self.prompt_state.open_commit_and_push();
                self.focus = Focus::PromptDialog;
            }
            Action::Push => {
                if let Some(ref ops) = self.git_ops {
                    match ops.push() {
                        Ok(msg) => self.error_message = Some(format!("Pushed: {}", msg.trim())),
                        Err(e) => self.error_message = Some(format!("Push failed: {}", e)),
                    }
                }
            }
            Action::Pull => {
                if let Some(ref ops) = self.git_ops {
                    match ops.pull() {
                        Ok(msg) => {
                            self.error_message = Some(format!("Pulled: {}", msg.trim()));
                            self.refresh_git();
                        }
                        Err(e) => self.error_message = Some(format!("Pull failed: {}", e)),
                    }
                }
            }
            Action::Stash => {
                if let Some(ref ops) = self.git_ops {
                    match ops.stash() {
                        Ok(msg) => self.error_message = Some(msg),
                        Err(e) => self.error_message = Some(format!("Stash failed: {}", e)),
                    }
                    self.refresh_git();
                }
            }
            Action::StashPop => {
                if let Some(ref ops) = self.git_ops {
                    match ops.stash_pop() {
                        Ok(msg) => self.error_message = Some(msg),
                        Err(e) => self.error_message = Some(format!("Stash pop failed: {}", e)),
                    }
                    self.refresh_git();
                }
            }
            Action::CreateBranch => {
                self.prompt_state.open_create_branch();
                self.focus = Focus::PromptDialog;
            }
            Action::CheckoutBranch(name) => {
                if let Some(ref ops) = self.git_ops {
                    if let Err(e) = ops.checkout_branch(&name) {
                        self.error_message = Some(format!("{}", e));
                    }
                    self.refresh_git();
                }
            }
            Action::BranchList => {
                // TODO: branch picker UI — for now show branches in error_message
                if let Some(ref ops) = self.git_ops {
                    match ops.branch_list() {
                        Ok(branches) => {
                            self.error_message = Some(
                                format!("Branches: {}", branches.join(", "))
                            );
                        }
                        Err(e) => self.error_message = Some(format!("{}", e)),
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_prompt_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.prompt_state.close();
                self.focus = Focus::GitStatus;
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if !self.prompt_state.input.is_empty() {
                    match self.prompt_state.mode {
                        PromptMode::Commit => {
                            if let Some(ref ops) = self.git_ops {
                                let msg = self.prompt_state.input.clone();
                                if let Err(e) = ops.commit(&msg) {
                                    self.error_message = Some(format!("{}", e));
                                }
                                self.refresh_git();
                            }
                        }
                        PromptMode::CommitAndPush => {
                            if let Some(ref ops) = self.git_ops {
                                let msg = self.prompt_state.input.clone();
                                match ops.commit(&msg) {
                                    Ok(()) => {
                                        match ops.push() {
                                            Ok(out) => self.error_message = Some(format!("Committed & pushed: {}", out.trim())),
                                            Err(e) => self.error_message = Some(format!("Committed but push failed: {}", e)),
                                        }
                                    }
                                    Err(e) => self.error_message = Some(format!("{}", e)),
                                }
                                self.refresh_git();
                            }
                        }
                        PromptMode::CreateBranch => {
                            if let Some(ref ops) = self.git_ops {
                                let name = self.prompt_state.input.clone();
                                if let Err(e) = ops.create_branch(&name) {
                                    self.error_message = Some(format!("{}", e));
                                } else {
                                    self.error_message = Some(format!("Switched to new branch '{}'", name));
                                }
                                self.refresh_git();
                            }
                        }
                        PromptMode::SendToClaude => {
                            let cmd = self.prompt_state.build_command();
                            self.pty.inject_input(&cmd).await?;
                            self.prompt_state.close();
                            self.focus = Focus::Pty;
                            return Ok(());
                        }
                    }
                } else if self.prompt_state.mode == PromptMode::SendToClaude
                    && !self.prompt_state.files.is_empty()
                {
                    // Allow sending files without prompt text
                    let cmd = self.prompt_state.build_command();
                    self.pty.inject_input(&cmd).await?;
                    self.prompt_state.close();
                    self.focus = Focus::Pty;
                    return Ok(());
                }
                self.prompt_state.close();
                self.focus = Focus::GitStatus;
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.prompt_state.delete_char();
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                self.prompt_state.move_cursor_left();
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                self.prompt_state.move_cursor_right();
            }
            (_, KeyCode::Char(c)) => {
                self.prompt_state.insert_char(c);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn resize_pty(&mut self, area: Rect) {
        let (cols, rows) = AppLayout::pty_inner_size(area);
        if cols > 0 && rows > 0 && area != self.last_pty_area {
            self.last_pty_area = area;
            self.emulator.set_size(rows, cols);
            let _ = self.pty.resize(cols, rows);
        }
    }
}

fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
