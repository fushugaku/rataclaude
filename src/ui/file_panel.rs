use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::filebrowser::panel::PanelState;

pub struct FilePanelWidget<'a> {
    pub state: &'a PanelState,
    pub focused: bool,
}

impl<'a> FilePanelWidget<'a> {
    pub fn new(state: &'a PanelState, focused: bool) -> Self {
        Self { state, focused }
    }

    fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{}B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1}K", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn format_time(time: &std::time::SystemTime) -> String {
        let duration = time.elapsed().unwrap_or_default();
        let secs = duration.as_secs();
        if secs < 60 {
            "now".to_string()
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else if secs < 86400 * 30 {
            format!("{}d", secs / 86400)
        } else if secs < 86400 * 365 {
            format!("{}mo", secs / (86400 * 30))
        } else {
            format!("{}y", secs / (86400 * 365))
        }
    }
}

impl Widget for FilePanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::Rgb(60, 60, 60)
        };

        let title = self.state.current_dir.to_string_lossy().to_string();
        // Truncate title if too long
        let max_title = area.width.saturating_sub(4) as usize;
        let display_title = if title.len() > max_title {
            format!("...{}", &title[title.len() - max_title + 3..])
        } else {
            title
        };

        let block = Block::default()
            .title(format!(" {} ", display_title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let viewport_height = inner.height as usize;

        // We need mutable access to scroll_offset but we only have &PanelState.
        // Calculate scroll offset locally based on cursor position.
        let mut scroll = self.state.scroll_offset;
        if self.state.cursor < scroll {
            scroll = self.state.cursor;
        } else if self.state.cursor >= scroll + viewport_height {
            scroll = self.state.cursor - viewport_height + 1;
        }

        let size_col_width = 7u16;
        let time_col_width = 4u16;
        let right_cols = size_col_width + time_col_width + 2; // 2 for spacing
        let name_width = inner.width.saturating_sub(right_cols);

        for (i, y) in (inner.y..inner.y + inner.height).enumerate() {
            let idx = scroll + i;
            if idx >= self.state.entries.len() {
                break;
            }

            let entry = &self.state.entries[idx];
            let is_cursor = idx == self.state.cursor;

            // Icon + name
            let icon = if entry.is_dir { "/" } else { " " };

            let name_color = if entry.is_dir {
                Color::Cyan
            } else {
                Color::White
            };

            let display_name = if entry.name.len() + 1 > name_width as usize {
                let trunc = name_width as usize - 2;
                format!("{}{}", &entry.name[..trunc.min(entry.name.len())], "~")
            } else {
                format!("{}{}", entry.name, icon)
            };

            let size_str = if entry.is_dir {
                "<DIR>".to_string()
            } else {
                Self::format_size(entry.size)
            };

            let time_str = entry
                .modified
                .as_ref()
                .map(Self::format_time)
                .unwrap_or_default();

            let bg = if is_cursor {
                Color::Rgb(50, 50, 80)
            } else {
                Color::Reset
            };

            // Name column
            let name_span = Span::styled(
                format!("{:<width$}", display_name, width = name_width as usize),
                Style::default().fg(name_color).bg(bg),
            );
            // Size column
            let size_span = Span::styled(
                format!("{:>width$}", size_str, width = size_col_width as usize),
                Style::default().fg(Color::Rgb(140, 140, 140)).bg(bg),
            );
            // Time column
            let time_span = Span::styled(
                format!(" {:>width$}", time_str, width = time_col_width as usize - 1),
                Style::default().fg(Color::Rgb(100, 100, 100)).bg(bg),
            );

            let line = Line::from(vec![name_span, size_span, time_span]);
            buf.set_line(inner.x, y, &line, inner.width);

            // If cursor, also apply bg to any remaining cells in the row
            if is_cursor {
                for x in inner.x..inner.x + inner.width {
                    buf[(x, y)].set_bg(bg);
                }
            }
        }

        // Show empty directory message
        if self.state.entries.is_empty() {
            let msg = " (empty) ";
            let style = Style::default()
                .fg(Color::Rgb(80, 80, 80))
                .add_modifier(Modifier::ITALIC);
            buf.set_string(inner.x + 1, inner.y, msg, style);
        }
    }
}
