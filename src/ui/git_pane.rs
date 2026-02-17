use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::git::diff::FileDiff;
use crate::git::status::FileStatus;
use crate::ui::diff_view::{self, DiffViewState};
use crate::ui::layout::AppLayout;
use crate::ui::status_list::{StatusListState, StatusListWidget};
use crate::app::Focus;

pub struct GitPane<'a> {
    pub files: &'a [FileStatus],
    pub diff: Option<&'a FileDiff>,
    pub branch: &'a str,
    pub focus: Focus,
    pub status_state: &'a mut StatusListState,
    pub diff_state: &'a DiffViewState,
}

impl Widget for GitPane<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let (status_area, diff_area) = AppLayout::split_right(area);

        let status_widget = StatusListWidget::new(
            self.files,
            self.focus == Focus::GitStatus,
            self.branch,
        );
        ratatui::widgets::StatefulWidget::render(
            status_widget,
            status_area,
            buf,
            &mut self.status_state,
        );

        diff_view::render_diff(
            self.diff,
            self.diff_state,
            self.focus == Focus::DiffView,
            diff_area,
            buf,
        );
    }
}
