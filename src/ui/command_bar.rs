use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::app::Focus;

pub struct CommandBar {
    focus: Focus,
    multi_select: bool,
}

impl CommandBar {
    pub fn new(focus: Focus, multi_select: bool) -> Self {
        Self { focus, multi_select }
    }

    fn key_hint(key: &str, desc: &str) -> Vec<Span<'static>> {
        vec![
            Span::styled(
                format!(" {} ", key),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", desc),
                Style::default().fg(Color::Gray),
            ),
        ]
    }
}

impl Widget for CommandBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans: Vec<Span> = vec![];

        // Global hints
        spans.extend(Self::key_hint("Tab", "focus"));
        spans.extend(Self::key_hint("C-q", "quit"));

        match self.focus {
            Focus::Pty => {
                spans.extend(Self::key_hint("C-\\", "resize"));
            }
            Focus::GitStatus => {
                spans.extend(Self::key_hint("j/k", "nav"));
                spans.extend(Self::key_hint("Spc", "stage"));
                spans.extend(Self::key_hint("s/S", "send"));
                spans.extend(Self::key_hint("c", "commit"));
                spans.extend(Self::key_hint("C", "commit+push"));
                spans.extend(Self::key_hint("p/P", "push/pull"));
                spans.extend(Self::key_hint("b/B", "branch/new"));
                spans.extend(Self::key_hint("z/Z", "stash/pop"));
            }
            Focus::DiffView => {
                spans.extend(Self::key_hint("j/k", "scroll"));
                spans.extend(Self::key_hint("J/K", "hunk"));
                spans.extend(Self::key_hint("Esc", "back"));
                spans.extend(Self::key_hint("s", "send"));
            }
            Focus::PromptDialog => {
                spans.extend(Self::key_hint("Enter", "confirm"));
                spans.extend(Self::key_hint("Esc", "cancel"));
            }
        }

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
