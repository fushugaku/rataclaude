use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

pub struct TerminalEmulator {
    parser: vt100::Parser,
}

impl TerminalEmulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows, cols, 1000),
        }
    }

    pub fn process(&mut self, data: &[u8]) {
        self.parser.process(data);
    }

    pub fn set_size(&mut self, rows: u16, cols: u16) {
        self.parser.set_size(rows, cols);
    }

    pub fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    pub fn mouse_enabled(&self) -> bool {
        self.parser.screen().mouse_protocol_mode()
            != vt100::MouseProtocolMode::None
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        let screen = self.parser.screen();

        for row in 0..area.height {
            for col in 0..area.width {
                let cell = screen.cell(row, col);
                if let Some(cell) = cell {
                    let x = area.x + col;
                    let y = area.y + row;

                    if x < area.right() && y < area.bottom() {
                        let mut style = Style::default();
                        style = style.fg(vt100_color_to_ratatui(cell.fgcolor()));
                        style = style.bg(vt100_color_to_ratatui(cell.bgcolor()));

                        let mut modifiers = Modifier::empty();
                        if cell.bold() {
                            modifiers |= Modifier::BOLD;
                        }
                        if cell.italic() {
                            modifiers |= Modifier::ITALIC;
                        }
                        if cell.underline() {
                            modifiers |= Modifier::UNDERLINED;
                        }
                        if cell.inverse() {
                            modifiers |= Modifier::REVERSED;
                        }
                        style = style.add_modifier(modifiers);

                        let ch = cell.contents();
                        let display_char = if ch.is_empty() { " " } else { &ch };
                        buf.set_string(x, y, display_char, style);
                    }
                }
            }
        }

        // Render cursor
        if focused {
            let cursor = screen.cursor_position();
            let cx = area.x + cursor.1;
            let cy = area.y + cursor.0;
            if cx < area.right() && cy < area.bottom() {
                let cell = buf.cell_mut((cx, cy));
                if let Some(cell) = cell {
                    cell.set_style(Style::default().add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}

fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
