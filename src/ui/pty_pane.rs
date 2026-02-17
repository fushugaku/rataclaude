use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

use crate::pty::terminal_emulator::TerminalEmulator;

pub struct PtyPane<'a> {
    emulator: &'a TerminalEmulator,
    focused: bool,
}

impl<'a> PtyPane<'a> {
    pub fn new(emulator: &'a TerminalEmulator, focused: bool) -> Self {
        Self { emulator, focused }
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
        self.emulator.render(inner, buf, self.focused);
    }
}
