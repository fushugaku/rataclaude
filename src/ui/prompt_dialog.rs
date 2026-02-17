use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptMode {
    SendToClaude,
    Commit,
    CommitAndPush,
    CreateBranch,
}

pub struct PromptDialogState {
    pub visible: bool,
    pub input: String,
    pub cursor_pos: usize,
    pub files: Vec<String>,
    pub mode: PromptMode,
}

impl PromptDialogState {
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            cursor_pos: 0,
            files: Vec::new(),
            mode: PromptMode::SendToClaude,
        }
    }

    pub fn open_send(&mut self, files: Vec<String>) {
        self.visible = true;
        self.input.clear();
        self.cursor_pos = 0;
        self.files = files;
        self.mode = PromptMode::SendToClaude;
    }

    pub fn open_commit(&mut self) {
        self.visible = true;
        self.input.clear();
        self.cursor_pos = 0;
        self.files.clear();
        self.mode = PromptMode::Commit;
    }

    pub fn open_commit_and_push(&mut self) {
        self.visible = true;
        self.input.clear();
        self.cursor_pos = 0;
        self.files.clear();
        self.mode = PromptMode::CommitAndPush;
    }

    pub fn open_create_branch(&mut self) {
        self.visible = true;
        self.input.clear();
        self.cursor_pos = 0;
        self.files.clear();
        self.mode = PromptMode::CreateBranch;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.input.clear();
        self.cursor_pos = 0;
        self.files.clear();
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor_pos -= prev;
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor_pos -= prev;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            let next = self.input[self.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor_pos += next;
        }
    }

    pub fn build_command(&self) -> String {
        let file_refs: Vec<String> = self.files.iter()
            .map(|f| format!("@{}", f))
            .collect();
        let files_str = file_refs.join(" ");

        if self.input.is_empty() {
            format!("{}\n", files_str)
        } else {
            format!("{} {}\n", self.input, files_str)
        }
    }
}

pub struct PromptDialog<'a> {
    state: &'a PromptDialogState,
}

impl<'a> PromptDialog<'a> {
    pub fn new(state: &'a PromptDialogState) -> Self {
        Self { state }
    }
}

impl Widget for PromptDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let (title, action_label) = match self.state.mode {
            PromptMode::SendToClaude => (" Send to Claude ", "send"),
            PromptMode::Commit => (" Commit ", "commit"),
            PromptMode::CommitAndPush => (" Commit & Push ", "commit+push"),
            PromptMode::CreateBranch => (" New Branch ", "create"),
        };

        // Center the dialog
        let dialog_width = area.width.min(60);
        let dialog_height = 8u16.min(area.height);
        let x = (area.width - dialog_width) / 2 + area.x;
        let y = (area.height - dialog_height) / 2 + area.y;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        Clear.render(dialog_area, buf);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        let has_files = !self.state.files.is_empty();
        let constraints = if has_files {
            vec![
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(1),
            ]
        } else {
            vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        if has_files {
            let files_text = self.state.files.iter()
                .map(|f| format!("@{}", f))
                .collect::<Vec<_>>()
                .join(" ");
            let files_line = Line::from(vec![
                Span::styled("Files: ", Style::default().fg(Color::DarkGray)),
                Span::styled(files_text, Style::default().fg(Color::Green)),
            ]);
            Paragraph::new(files_line).render(chunks[0], buf);
        } else {
            let placeholder = match self.state.mode {
                PromptMode::Commit | PromptMode::CommitAndPush => "Enter commit message:",
                PromptMode::CreateBranch => "Enter branch name:",
                _ => "",
            };
            let label = Line::from(Span::styled(placeholder, Style::default().fg(Color::DarkGray)));
            Paragraph::new(label).render(chunks[0], buf);
        }

        // Input
        let input_line = Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan)),
            Span::raw(&self.state.input),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]);
        Paragraph::new(input_line).render(chunks[1], buf);

        // Help
        let help = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(format!(" {}  ", action_label)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]);
        Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .render(chunks[2], buf);
    }
}
