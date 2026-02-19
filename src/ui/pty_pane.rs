use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

use crate::app::PtySelection;
use crate::pty::terminal_emulator::TerminalEmulator;

pub struct PtyPane<'a> {
    emulator: &'a TerminalEmulator,
    focused: bool,
    selection: &'a PtySelection,
}

impl<'a> PtyPane<'a> {
    pub fn new(emulator: &'a TerminalEmulator, focused: bool, selection: &'a PtySelection) -> Self {
        Self { emulator, focused, selection }
    }
}

impl Widget for PtyPane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(if self.focused {
                ratatui::style::Style::default().fg(ratatui::style::Color::Cyan)
            } else {
                ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray)
            });

        let inner = block.inner(area);
        block.render(area, buf);
        self.emulator.render(inner, buf, self.focused, self.selection);
    }
}
