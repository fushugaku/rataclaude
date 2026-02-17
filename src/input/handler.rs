use crossterm::event::KeyEvent;

use crate::action::Action;
use crate::app::Focus;
use crate::input::keymap;

pub fn handle_key(key: KeyEvent, focus: Focus) -> Option<Action> {
    keymap::map_key(key, focus)
}
