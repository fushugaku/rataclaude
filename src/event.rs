use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};

use crate::git::status::FileStatus;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    PtyOutput(Vec<u8>),
    PtyExited,
    Tick,
    GitRefresh,
    /// Async git status result from background thread
    GitStatusUpdate(Vec<FileStatus>, String),
    /// Terminal focus gained (from real terminal)
    FocusGained,
    /// Terminal focus lost (from real terminal)
    FocusLost,
}

impl From<CrosstermEvent> for AppEvent {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => AppEvent::Key(key),
            CrosstermEvent::Key(_) => AppEvent::Tick, // ignore release/repeat
            CrosstermEvent::Mouse(mouse) => AppEvent::Mouse(mouse),
            CrosstermEvent::Resize(w, h) => AppEvent::Resize(w, h),
            CrosstermEvent::FocusGained => AppEvent::FocusGained,
            CrosstermEvent::FocusLost => AppEvent::FocusLost,
            _ => AppEvent::Tick,
        }
    }
}
