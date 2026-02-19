use std::path::Path;

use super::panel::PanelState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSide {
    Left,
    Right,
}

pub struct FileBrowserState {
    pub left: PanelState,
    pub right: PanelState,
    pub active_panel: PanelSide,
}

impl FileBrowserState {
    pub fn new(workdir: &Path) -> Self {
        Self {
            left: PanelState::new(workdir),
            right: PanelState::new(workdir),
            active_panel: PanelSide::Left,
        }
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        match self.active_panel {
            PanelSide::Left => &mut self.left,
            PanelSide::Right => &mut self.right,
        }
    }

    pub fn inactive_panel(&self) -> &PanelState {
        match self.active_panel {
            PanelSide::Left => &self.right,
            PanelSide::Right => &self.left,
        }
    }

    pub fn switch_panel(&mut self) {
        self.active_panel = match self.active_panel {
            PanelSide::Left => PanelSide::Right,
            PanelSide::Right => PanelSide::Left,
        };
    }
}
