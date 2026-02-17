use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::git::diff::{DiffLineKind, FileDiff};
use crate::ui::syntax::{self, HighlightSpan};

// ── True-color palette (looks great on Ghostty) ─────────────────────
const ADD_BG: Color = Color::Rgb(22, 39, 28);
const ADD_FG: Color = Color::Rgb(86, 209, 108);
const ADD_GUTTER_FG: Color = Color::Rgb(60, 150, 80);

const DEL_BG: Color = Color::Rgb(50, 22, 22);
const DEL_FG: Color = Color::Rgb(235, 100, 95);
const DEL_GUTTER_FG: Color = Color::Rgb(170, 70, 65);

const CTX_FG: Color = Color::Rgb(140, 140, 140);
const CTX_GUTTER_FG: Color = Color::Rgb(80, 80, 80);

const HUNK_BG: Color = Color::Rgb(30, 35, 50);
const HUNK_FG: Color = Color::Rgb(110, 150, 220);
const HUNK_ACCENT: Color = Color::Rgb(70, 100, 170);

const GUTTER_BG: Color = Color::Rgb(25, 25, 30);
const GUTTER_SEP: Color = Color::Rgb(50, 50, 60);

const EMPTY_FG: Color = Color::Rgb(90, 90, 110);
const BORDER_FOCUSED: Color = Color::Rgb(100, 180, 255);
const BORDER_UNFOCUSED: Color = Color::Rgb(55, 55, 65);

const SCROLLBAR_TRACK: Color = Color::Rgb(35, 35, 40);
const SCROLLBAR_THUMB: Color = Color::Rgb(90, 90, 110);

const CURSOR_BG: Color = Color::Rgb(45, 50, 65);
const SELECT_BG: Color = Color::Rgb(40, 55, 80);

pub struct DiffViewState {
    pub scroll: u16,
    pub h_scroll: u16,
    pub cursor: usize,
    pub select_anchor: Option<usize>,
    pub file_path: Option<String>,
    /// Cached syntax-highlighted spans for each line in the diff.
    /// Recomputed only when the diff changes (set_file / update_highlight_cache).
    pub highlight_cache: Vec<Vec<HighlightSpan>>,
}

impl DiffViewState {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            h_scroll: 0,
            cursor: 0,
            select_anchor: None,
            file_path: None,
            highlight_cache: Vec::new(),
        }
    }

    pub fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        // Keep cursor in view
        if self.cursor < self.scroll as usize {
            self.scroll = self.cursor as u16;
        }
    }

    pub fn cursor_down(&mut self, max: usize) {
        if self.cursor + 1 < max {
            self.cursor += 1;
        }
    }

    /// Ensure cursor is scrolled into view given viewport height
    pub fn ensure_visible(&mut self, viewport_h: u16) {
        let bottom = self.scroll as usize + viewport_h as usize;
        if self.cursor >= bottom {
            self.scroll = (self.cursor + 1).saturating_sub(viewport_h as usize) as u16;
        }
        if self.cursor < self.scroll as usize {
            self.scroll = self.cursor as u16;
        }
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: u16, max: u16) {
        self.scroll = (self.scroll + amount).min(max);
    }

    pub fn scroll_left(&mut self, amount: u16) {
        self.h_scroll = self.h_scroll.saturating_sub(amount);
    }

    pub fn scroll_right(&mut self, amount: u16) {
        self.h_scroll = self.h_scroll.saturating_add(amount);
    }

    pub fn toggle_select(&mut self) {
        if self.select_anchor.is_some() {
            self.select_anchor = None;
        } else {
            self.select_anchor = Some(self.cursor);
        }
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.select_anchor.map(|anchor| {
            let start = anchor.min(self.cursor);
            let end = anchor.max(self.cursor);
            (start, end)
        })
    }

    pub fn clear_select(&mut self) {
        self.select_anchor = None;
    }

    pub fn reset(&mut self) {
        self.scroll = 0;
        self.h_scroll = 0;
        self.cursor = 0;
        self.select_anchor = None;
        self.file_path = None;
        self.highlight_cache.clear();
    }

    pub fn set_file(&mut self, path: &str) {
        if self.file_path.as_deref() != Some(path) {
            self.file_path = Some(path.to_string());
            self.scroll = 0;
            self.h_scroll = 0;
            self.cursor = 0;
            self.select_anchor = None;
            self.highlight_cache.clear();
        }
    }

    /// Pre-compute syntax highlighting for all lines in a diff.
    /// Call this when the diff content changes.
    pub fn update_highlight_cache(&mut self, diff: &FileDiff) {
        let all_lines = diff.all_lines();
        let lines_for_highlight: Vec<(String, bool)> = all_lines
            .iter()
            .map(|line| {
                let content = line.content.trim_end_matches('\n').to_string();
                let visible = line.kind != DiffLineKind::HunkHeader;
                (content, visible)
            })
            .collect();
        self.highlight_cache = syntax::highlight_diff_lines(&diff.path, &lines_for_highlight);
    }
}

pub struct DiffViewWidget<'a> {
    diff: Option<&'a FileDiff>,
    focused: bool,
}

impl<'a> DiffViewWidget<'a> {
    pub fn new(diff: Option<&'a FileDiff>, focused: bool) -> Self {
        Self { diff, focused }
    }
}

impl Widget for DiffViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_diff(self.diff, &DiffViewState::new(), self.focused, area, buf);
    }
}

/// Render diff with scroll state — the main entry point
pub fn render_diff(
    diff: Option<&FileDiff>,
    state: &DiffViewState,
    focused: bool,
    area: Rect,
    buf: &mut Buffer,
) {
    let border_style = if focused {
        Style::default().fg(BORDER_FOCUSED)
    } else {
        Style::default().fg(BORDER_UNFOCUSED)
    };

    let (title, stats) = match &diff {
        Some(d) => {
            let (adds, dels) = count_changes(d);
            let stats_str = if adds > 0 || dels > 0 {
                format!(" +{} -{} ", adds, dels)
            } else {
                String::new()
            };
            (format!(" {} ", d.path), stats_str)
        }
        None => (" diff ".to_string(), String::new()),
    };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(&title, border_style),
            Span::styled(
                &stats,
                Style::default()
                    .fg(if stats.contains('+') { ADD_FG } else { CTX_FG }),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    match diff {
        Some(diff) => render_diff_lines(diff, state, focused, inner, buf),
        None => render_empty(inner, buf),
    }
}

fn render_empty(area: Rect, buf: &mut Buffer) {
    let messages = [
        "",
        "   No diff selected",
        "",
        "   Select a file and press Enter",
        "   to view changes",
    ];

    for (i, msg) in messages.iter().enumerate() {
        let y = area.y + (area.height / 3) + i as u16;
        if y < area.bottom() {
            buf.set_string(area.x, y, msg, Style::default().fg(EMPTY_FG));
        }
    }
}

fn render_diff_lines(diff: &FileDiff, state: &DiffViewState, focused: bool, area: Rect, buf: &mut Buffer) {
    let all_lines = diff.all_lines();
    let total = all_lines.len();
    let scroll = state.scroll as usize;

    // Gutter width: "NNNN │ NNNN │ " = ~13 chars.  Adapt to max line numbers.
    let gutter_w: u16 = 13;
    let content_x = area.x + gutter_w;
    let _content_w = area.width.saturating_sub(gutter_w + 1); // 1 for scrollbar
    let scrollbar_x = area.right().saturating_sub(1);

    for row in 0..area.height {
        let line_idx = scroll + row as usize;
        let y = area.y + row;

        if line_idx >= total {
            // Fill remaining rows with gutter background
            for x in area.x..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ');
                    cell.set_style(Style::default().bg(GUTTER_BG));
                }
            }
            continue;
        }

        let line = &all_lines[line_idx];

        match line.kind {
            DiffLineKind::HunkHeader => {
                render_hunk_header(line, area.x, y, area.width.saturating_sub(1), buf);
            }
            _ => {
                let (line_bg, line_fg, gutter_fg, prefix_char) = match line.kind {
                    DiffLineKind::Addition => (ADD_BG, ADD_FG, ADD_GUTTER_FG, '+'),
                    DiffLineKind::Deletion => (DEL_BG, DEL_FG, DEL_GUTTER_FG, '-'),
                    _ => (Color::Reset, CTX_FG, CTX_GUTTER_FG, ' '),
                };

                // ── Gutter: old lineno ──
                let old_str = match line.old_lineno {
                    Some(n) => format!("{:>4}", n),
                    None => "    ".to_string(),
                };
                buf.set_string(
                    area.x,
                    y,
                    &old_str,
                    Style::default().fg(gutter_fg).bg(GUTTER_BG),
                );

                // Separator
                buf.set_string(
                    area.x + 4,
                    y,
                    " \u{2502} ",
                    Style::default().fg(GUTTER_SEP).bg(GUTTER_BG),
                );

                // ── Gutter: new lineno ──
                let new_str = match line.new_lineno {
                    Some(n) => format!("{:>4}", n),
                    None => "    ".to_string(),
                };
                buf.set_string(
                    area.x + 7,
                    y,
                    &new_str,
                    Style::default().fg(gutter_fg).bg(GUTTER_BG),
                );

                // Separator
                buf.set_string(
                    area.x + 11,
                    y,
                    " \u{2502}",
                    Style::default().fg(GUTTER_SEP).bg(GUTTER_BG),
                );

                // ── Prefix (+/-/space) ──
                let prefix_style = Style::default()
                    .fg(line_fg)
                    .bg(line_bg)
                    .add_modifier(Modifier::BOLD);
                buf.set_string(content_x, y, &prefix_char.to_string(), prefix_style);

                // ── Content with syntax highlighting + horizontal scroll ──
                let content = line.content.trim_end_matches('\n');
                let h_off = state.h_scroll as usize;
                let content_w = (scrollbar_x.saturating_sub(content_x + 1)) as usize;
                // Use cached highlights if available, otherwise empty
                let empty_spans = Vec::new();
                let spans = if line_idx < state.highlight_cache.len() {
                    &state.highlight_cache[line_idx]
                } else {
                    &empty_spans
                };

                let mut cx = content_x + 1;
                if spans.is_empty() {
                    // Fallback: render plain with h_scroll (char-aware)
                    let visible: String = content.chars()
                        .skip(h_off)
                        .take(content_w)
                        .collect();
                    buf.set_string(
                        cx,
                        y,
                        &visible,
                        Style::default().fg(line_fg).bg(line_bg),
                    );
                    cx += visible.chars().count() as u16;
                } else {
                    // Walk through spans using char counts, not byte counts
                    let mut char_pos: usize = 0;
                    for span in spans {
                        let span_chars: usize = span.text.chars().count();
                        let span_end = char_pos + span_chars;

                        if span_end <= h_off {
                            char_pos = span_end;
                            continue;
                        }

                        let skip = if h_off > char_pos { h_off - char_pos } else { 0 };
                        let rendered = (cx - (content_x + 1)) as usize;
                        let remaining_w = content_w.saturating_sub(rendered);
                        if remaining_w == 0 {
                            break;
                        }
                        let visible: String = span.text.chars()
                            .skip(skip)
                            .take(remaining_w)
                            .collect();

                        let mut style = Style::default().fg(span.fg).bg(line_bg);
                        if span.bold {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        if span.italic {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        let vis_chars = visible.chars().count() as u16;
                        buf.set_string(cx, y, &visible, style);
                        cx += vis_chars;

                        char_pos = span_end;
                    }
                }

                // Fill remaining width with background color
                for x in cx..scrollbar_x {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ');
                        cell.set_style(Style::default().bg(line_bg));
                    }
                }
            }
        }

        // Scrollbar column
        render_scrollbar_cell(scrollbar_x, y, row, area.height, scroll, total, buf);

        // Cursor / selection overlay
        if line_idx < total {
            let is_cursor = line_idx == state.cursor && focused;
            let is_selected = state.selection_range()
                .map(|(s, e)| line_idx >= s && line_idx <= e)
                .unwrap_or(false);

            if is_cursor || is_selected {
                let overlay_bg = if is_cursor { CURSOR_BG } else { SELECT_BG };
                for x in area.x..scrollbar_x {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(cell.style().bg(overlay_bg));
                    }
                }
            }
        }
    }
}

fn render_hunk_header(
    line: &crate::git::diff::DiffLine,
    x: u16,
    y: u16,
    width: u16,
    buf: &mut Buffer,
) {
    let content = line.content.trim_end_matches('\n');

    // Fill entire line with hunk background
    for col in x..x + width {
        if let Some(cell) = buf.cell_mut((col, y)) {
            cell.set_char(' ');
            cell.set_style(Style::default().bg(HUNK_BG));
        }
    }

    // Leading accent bar
    buf.set_string(
        x,
        y,
        " \u{2500}\u{2500}\u{2500} ",
        Style::default().fg(HUNK_ACCENT).bg(HUNK_BG),
    );

    // Hunk header text (e.g. @@ -10,5 +10,7 @@ fn foo)
    // Parse out the function name if present
    let (range_part, fn_part) = if let Some(idx) = content.find("@@").and_then(|first| {
        content[first + 2..].find("@@").map(|second| first + 2 + second + 2)
    }) {
        (&content[..idx], content[idx..].trim())
    } else {
        (content, "")
    };

    buf.set_string(
        x + 5,
        y,
        range_part.trim(),
        Style::default()
            .fg(HUNK_FG)
            .bg(HUNK_BG)
            .add_modifier(Modifier::BOLD),
    );

    if !fn_part.is_empty() {
        let offset = 5 + range_part.trim().len() as u16 + 1;
        buf.set_string(
            x + offset,
            y,
            fn_part,
            Style::default()
                .fg(HUNK_ACCENT)
                .bg(HUNK_BG)
                .add_modifier(Modifier::ITALIC),
        );
    }

    // Trailing accent
    let end_x = x + width;
    if end_x > x + 3 {
        buf.set_string(
            end_x.saturating_sub(4),
            y,
            " \u{2500}\u{2500}\u{2500}",
            Style::default().fg(HUNK_ACCENT).bg(HUNK_BG),
        );
    }
}

fn render_scrollbar_cell(
    x: u16,
    y: u16,
    row: u16,
    viewport_height: u16,
    scroll: usize,
    total: usize,
    buf: &mut Buffer,
) {
    if total <= viewport_height as usize {
        // No scrollbar needed — fill with subtle track
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(' ');
            cell.set_style(Style::default().bg(SCROLLBAR_TRACK));
        }
        return;
    }

    let thumb_height = ((viewport_height as f64 / total as f64) * viewport_height as f64)
        .max(1.0) as u16;
    let max_scroll = total.saturating_sub(viewport_height as usize);
    let thumb_offset = if max_scroll > 0 {
        ((scroll as f64 / max_scroll as f64) * (viewport_height - thumb_height) as f64) as u16
    } else {
        0
    };

    let is_thumb = row >= thumb_offset && row < thumb_offset + thumb_height;

    if let Some(cell) = buf.cell_mut((x, y)) {
        if is_thumb {
            cell.set_char('\u{2588}'); // Full block
            cell.set_style(Style::default().fg(SCROLLBAR_THUMB).bg(SCROLLBAR_TRACK));
        } else {
            cell.set_char('\u{2591}'); // Light shade
            cell.set_style(Style::default().fg(SCROLLBAR_TRACK).bg(Color::Reset));
        }
    }
}

fn count_changes(diff: &FileDiff) -> (usize, usize) {
    let mut adds = 0;
    let mut dels = 0;
    for line in diff.all_lines() {
        match line.kind {
            DiffLineKind::Addition => adds += 1,
            DiffLineKind::Deletion => dels += 1,
            _ => {}
        }
    }
    (adds, dels)
}
