use ratatui::style::Color;
use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet};

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME: OnceLock<Theme> = OnceLock::new();

const SWIFT_SYNTAX: &str = include_str!("syntaxes/Swift.sublime-syntax");

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| {
        let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
        if let Ok(swift) = SyntaxDefinition::load_from_str(SWIFT_SYNTAX, true, None) {
            builder.add(swift);
        }
        builder.build()
    })
}

fn theme() -> &'static Theme {
    THEME.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        ts.themes["base16-ocean.dark"].clone()
    })
}

/// A styled fragment of text with fg color and optional modifiers.
pub struct HighlightSpan {
    pub text: String,
    pub fg: Color,
    pub bold: bool,
    pub italic: bool,
}

/// Map extensions not in the default set to a close-enough syntax.
fn extension_fallback(ext: &str) -> Option<&'static str> {
    match ext {
        "ts" | "mts" | "cts" => Some("js"),
        "tsx" | "jsx" => Some("js"),
        "kt" | "kts" => Some("java"),
        "vue" | "svelte" => Some("html"),
        "zig" => Some("c"),
        "dockerfile" => Some("sh"),
        _ => None,
    }
}

/// Find the syntect syntax for a given file path.
fn syntax_for_path(path: &str) -> &'static SyntaxReference {
    let ss = syntax_set();
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    ss.find_syntax_by_extension(ext)
        .or_else(|| {
            // Try matching by first component of filename (e.g. Dockerfile, Makefile)
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            ss.find_syntax_by_extension(name)
        })
        .or_else(|| {
            // Fallback: map similar languages
            extension_fallback(ext).and_then(|fb| ss.find_syntax_by_extension(fb))
        })
        .unwrap_or_else(|| ss.find_syntax_plain_text())
}

/// Highlight a single line of code. Returns spans with foreground colors.
/// Each span is a fragment of the line with its own color.
pub fn highlight_line(path: &str, content: &str) -> Vec<HighlightSpan> {
    let ss = syntax_set();
    let syntax = syntax_for_path(path);
    let mut h = HighlightLines::new(syntax, theme());

    // Ensure content has a newline for syntect (it expects newline-terminated lines)
    let line = if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n", content)
    };

    match h.highlight_line(&line, ss) {
        Ok(ranges) => ranges
            .into_iter()
            .map(|(style, text)| {
                // Strip trailing newline from the last span
                let text = text.trim_end_matches('\n').to_string();
                HighlightSpan {
                    text,
                    fg: syntect_to_ratatui_color(style),
                    bold: style.font_style.contains(FontStyle::BOLD),
                    italic: style.font_style.contains(FontStyle::ITALIC),
                }
            })
            .filter(|s| !s.text.is_empty())
            .collect(),
        Err(_) => vec![HighlightSpan {
            text: content.to_string(),
            fg: Color::Reset,
            bold: false,
            italic: false,
        }],
    }
}

/// Pre-highlight all lines in a diff sequentially, maintaining parser state
/// across lines for better multi-line construct handling.
pub fn highlight_diff_lines(path: &str, lines: &[(String, bool)]) -> Vec<Vec<HighlightSpan>> {
    let ss = syntax_set();
    let syntax = syntax_for_path(path);
    let mut h = HighlightLines::new(syntax, theme());

    lines
        .iter()
        .map(|(content, visible)| {
            let line = if content.ends_with('\n') {
                content.clone()
            } else {
                format!("{}\n", content)
            };

            match h.highlight_line(&line, ss) {
                Ok(ranges) => {
                    if !visible {
                        // Deleted lines: we ran the highlighter to keep state,
                        // but we don't need the output
                        return vec![];
                    }
                    ranges
                        .into_iter()
                        .map(|(style, text)| {
                            let text = text.trim_end_matches('\n').to_string();
                            HighlightSpan {
                                text,
                                fg: syntect_to_ratatui_color(style),
                                bold: style.font_style.contains(FontStyle::BOLD),
                                italic: style.font_style.contains(FontStyle::ITALIC),
                            }
                        })
                        .filter(|s| !s.text.is_empty())
                        .collect()
                }
                Err(_) => {
                    if !visible {
                        return vec![];
                    }
                    vec![HighlightSpan {
                        text: content.clone(),
                        fg: Color::Reset,
                        bold: false,
                        italic: false,
                    }]
                }
            }
        })
        .collect()
}

fn syntect_to_ratatui_color(style: Style) -> Color {
    Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}
