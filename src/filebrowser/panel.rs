use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

pub struct PanelState {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub show_hidden: bool,
}

impl PanelState {
    pub fn new(path: &Path) -> Self {
        let mut panel = Self {
            current_dir: path.to_path_buf(),
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            show_hidden: false,
        };
        panel.refresh();
        panel
    }

    pub fn refresh(&mut self) {
        self.entries.clear();

        let read_dir = match std::fs::read_dir(&self.current_dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };

        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            if !self.show_hidden && name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata.as_ref().and_then(|m| m.modified().ok());

            self.entries.push(DirEntry {
                name,
                path: entry.path(),
                is_dir,
                size,
                modified,
            });
        }

        // Sort: directories first, then alphabetical (case-insensitive)
        self.entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        // Clamp cursor
        if !self.entries.is_empty() {
            self.cursor = self.cursor.min(self.entries.len() - 1);
        } else {
            self.cursor = 0;
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if !self.entries.is_empty() && self.cursor < self.entries.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.cursor = self.cursor.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, page_size: usize) {
        if !self.entries.is_empty() {
            self.cursor = (self.cursor + page_size).min(self.entries.len() - 1);
        }
    }

    pub fn enter(&mut self) {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir {
                let new_dir = entry.path.clone();
                self.current_dir = new_dir;
                self.cursor = 0;
                self.scroll_offset = 0;
                self.refresh();
            }
        }
    }

    pub fn parent_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let old_name = self.current_dir.file_name()
                .map(|n| n.to_string_lossy().to_string());
            self.current_dir = parent.to_path_buf();
            self.cursor = 0;
            self.scroll_offset = 0;
            self.refresh();

            // Try to position cursor on the directory we came from
            if let Some(name) = old_name {
                if let Some(pos) = self.entries.iter().position(|e| e.name == name) {
                    self.cursor = pos;
                }
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.cursor)
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.refresh();
    }

    /// Ensure cursor is visible in viewport
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor - viewport_height + 1;
        }
    }
}
