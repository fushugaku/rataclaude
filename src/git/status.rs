#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatusKind {
    New,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
    Untracked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageState {
    Unstaged,
    Staged,
    Partial,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub kind: FileStatusKind,
    pub stage_state: StageState,
    pub index_status: Option<FileStatusKind>,
    pub worktree_status: Option<FileStatusKind>,
}

impl FileStatus {
    pub fn icon(&self) -> &str {
        match self.stage_state {
            StageState::Staged => "✓",
            StageState::Partial => "±",
            StageState::Unstaged => " ",
        }
    }

    pub fn kind_icon(&self) -> &str {
        let kind = self.worktree_status.as_ref()
            .or(self.index_status.as_ref())
            .unwrap_or(&self.kind);
        match kind {
            FileStatusKind::New => "A",
            FileStatusKind::Modified => "M",
            FileStatusKind::Deleted => "D",
            FileStatusKind::Renamed => "R",
            FileStatusKind::Typechange => "T",
            FileStatusKind::Conflicted => "C",
            FileStatusKind::Untracked => "?",
        }
    }

    pub fn kind_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        let kind = self.worktree_status.as_ref()
            .or(self.index_status.as_ref())
            .unwrap_or(&self.kind);
        match kind {
            FileStatusKind::New | FileStatusKind::Untracked => Color::Green,
            FileStatusKind::Modified | FileStatusKind::Typechange => Color::Yellow,
            FileStatusKind::Deleted => Color::Red,
            FileStatusKind::Renamed => Color::Cyan,
            FileStatusKind::Conflicted => Color::Magenta,
        }
    }
}
