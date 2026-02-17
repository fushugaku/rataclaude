use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::git::status::FileStatus;

pub struct StatusListState {
    pub list_state: ListState,
    pub selected: std::collections::HashSet<usize>,
    pub multi_select: bool,
}

impl StatusListState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            selected: std::collections::HashSet::new(),
            multi_select: false,
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn move_up(&mut self, len: usize) {
        if len == 0 { return; }
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i == 0 { len - 1 } else { i - 1 };
        self.list_state.select(Some(next));
    }

    pub fn move_down(&mut self, len: usize) {
        if len == 0 { return; }
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i >= len - 1 { 0 } else { i + 1 };
        self.list_state.select(Some(next));
    }

    pub fn toggle_select(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if self.selected.contains(&i) {
                self.selected.remove(&i);
            } else {
                self.selected.insert(i);
            }
        }
    }

    pub fn toggle_multi_select(&mut self) {
        self.multi_select = !self.multi_select;
        if !self.multi_select {
            self.selected.clear();
        }
    }

    pub fn selected_files<'a>(&self, files: &'a [FileStatus]) -> Vec<&'a FileStatus> {
        if self.multi_select && !self.selected.is_empty() {
            self.selected.iter()
                .filter_map(|&i| files.get(i))
                .collect()
        } else if let Some(i) = self.list_state.selected() {
            files.get(i).into_iter().collect()
        } else {
            vec![]
        }
    }
}

pub struct StatusListWidget<'a> {
    files: &'a [FileStatus],
    focused: bool,
    title: String,
}

impl<'a> StatusListWidget<'a> {
    pub fn new(files: &'a [FileStatus], focused: bool, branch: &str) -> Self {
        Self {
            files,
            focused,
            title: format!(" {} ({}) ", branch, files.len()),
        }
    }
}

impl StatefulWidget for StatusListWidget<'_> {
    type State = StatusListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(border_style);

        let items: Vec<ListItem> = self.files.iter().enumerate().map(|(i, file)| {
            let is_selected = state.selected.contains(&i);
            let marker = if state.multi_select {
                if is_selected { "● " } else { "○ " }
            } else {
                ""
            };

            let line = Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} ", file.icon()),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("{} ", file.kind_icon()),
                    Style::default().fg(file.kind_color()),
                ),
                Span::raw(&file.path),
            ]);

            ListItem::new(line)
        }).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
