use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode().context("enable_raw_mode (is stdin a TTY?)")?;
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::event::PushKeyboardEnhancementFlags(
            crossterm::event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        )
    )
    .context("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).context("create terminal")?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    let mut stdout = io::stdout();
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
        stdout,
        crossterm::event::PopKeyboardEnhancementFlags,
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    );
    Ok(())
}
