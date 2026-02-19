use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::filebrowser::state::{FileBrowserState, PanelSide};

use super::file_panel::FilePanelWidget;

pub struct FileBrowserPane<'a> {
    pub state: &'a FileBrowserState,
}

impl<'a> FileBrowserPane<'a> {
    pub fn new(state: &'a FileBrowserState) -> Self {
        Self { state }
    }
}

impl Widget for FileBrowserPane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        let left = FilePanelWidget::new(
            &self.state.left,
            self.state.active_panel == PanelSide::Left,
        );
        left.render(chunks[0], buf);

        let right = FilePanelWidget::new(
            &self.state.right,
            self.state.active_panel == PanelSide::Right,
        );
        right.render(chunks[1], buf);
    }
}
