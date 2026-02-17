#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
    HunkHeader,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
}

impl FileDiff {
    pub fn all_lines(&self) -> Vec<&DiffLine> {
        self.hunks.iter().flat_map(|h| h.lines.iter()).collect()
    }

    pub fn total_lines(&self) -> usize {
        self.hunks.iter().map(|h| h.lines.len()).sum()
    }
}

impl DiffLine {
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.kind {
            DiffLineKind::Addition => Color::Green,
            DiffLineKind::Deletion => Color::Red,
            DiffLineKind::HunkHeader => Color::Cyan,
            DiffLineKind::Context => Color::Reset,
        }
    }

    pub fn prefix(&self) -> &str {
        match self.kind {
            DiffLineKind::Addition => "+",
            DiffLineKind::Deletion => "-",
            DiffLineKind::HunkHeader => "@",
            DiffLineKind::Context => " ",
        }
    }
}
