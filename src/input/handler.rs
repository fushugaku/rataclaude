use crossterm::event::KeyEvent;

use crate::action::{Action, ActiveTab};
use crate::app::Focus;
use crate::input::keymap;

pub fn handle_key(key: KeyEvent, focus: Focus, active_tab: ActiveTab) -> Option<Action> {
    keymap::map_key(key, focus, active_tab)
}
