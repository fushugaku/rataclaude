use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub split_percent: u16,
}

impl AppLayout {
    pub fn new() -> Self {
        Self { split_percent: 60 }
    }

    pub fn adjust(&mut self, delta: i16) {
        let new_val = self.split_percent as i16 + delta;
        self.split_percent = new_val.clamp(20, 80) as u16;
    }

    pub fn split(&self, area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.split_percent),
                Constraint::Percentage(100 - self.split_percent),
            ])
            .split(area);
        (chunks[0], chunks[1])
    }

    /// Split the right pane into status list (top) and diff view (bottom)
    pub fn split_right(area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(area);
        (chunks[0], chunks[1])
    }

    /// Get the bottom bar area and main area
    pub fn with_command_bar(area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);
        (chunks[0], chunks[1])
    }

    /// Tab bar (1 line) + content area + command bar (1 line)
    pub fn with_tab_and_command_bar(area: Rect) -> (Rect, Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);
        (chunks[0], chunks[1], chunks[2])
    }

    /// Inner area for the PTY (excluding border)
    pub fn pty_inner_size(area: Rect) -> (u16, u16) {
        // Account for right border (1 col)
        let cols = area.width.saturating_sub(1);
        let rows = area.height;
        (cols, rows)
    }
}
