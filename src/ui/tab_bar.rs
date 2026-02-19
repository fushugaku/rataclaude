use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::action::ActiveTab;

pub struct TabBar {
    active: ActiveTab,
}

impl TabBar {
    pub fn new(active: ActiveTab) -> Self {
        Self { active }
    }

    fn tab_span(label: &str, is_active: bool) -> Vec<Span<'static>> {
        if is_active {
            vec![
                Span::styled(
                    " ".to_string(),
                    Style::default().bg(Color::DarkGray),
                ),
                Span::styled(
                    label.to_string(),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ".to_string(),
                    Style::default().bg(Color::DarkGray),
                ),
            ]
        } else {
            vec![
                Span::styled(
                    " ".to_string(),
                    Style::default().bg(Color::Rgb(30, 30, 30)),
                ),
                Span::styled(
                    label.to_string(),
                    Style::default()
                        .fg(Color::Rgb(120, 120, 120))
                        .bg(Color::Rgb(30, 30, 30)),
                ),
                Span::styled(
                    " ".to_string(),
                    Style::default().bg(Color::Rgb(30, 30, 30)),
                ),
            ]
        }
    }
}

impl Widget for TabBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Fill background
        for x in area.x..area.x + area.width {
            buf[(x, area.y)]
                .set_char(' ')
                .set_style(Style::default().bg(Color::Rgb(20, 20, 20)));
        }

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::Rgb(20, 20, 20)),
        ));
        spans.extend(Self::tab_span(
            "Claude Code",
            self.active == ActiveTab::ClaudeCode,
        ));
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::Rgb(20, 20, 20)),
        ));
        spans.extend(Self::tab_span(
            "Files",
            self.active == ActiveTab::FileBrowser,
        ));

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
