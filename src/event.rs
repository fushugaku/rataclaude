use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    PtyOutput(Vec<u8>),
    PtyExited,
    Tick,
    GitRefresh,
}

impl From<CrosstermEvent> for AppEvent {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => AppEvent::Key(key),
            CrosstermEvent::Key(_) => AppEvent::Tick, // ignore release/repeat
            CrosstermEvent::Mouse(mouse) => AppEvent::Mouse(mouse),
            CrosstermEvent::Resize(w, h) => AppEvent::Resize(w, h),
            _ => AppEvent::Tick,
        }
    }
}
