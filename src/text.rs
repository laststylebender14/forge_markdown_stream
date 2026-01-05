//! Text wrapping utilities with ANSI code awareness.

use unicode_width::UnicodeWidthChar;

/// Calculate visible length of a string (excluding ANSI escape codes).
/// Uses Unicode width to properly handle wide characters like emojis.
pub fn visible_length(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if in_escape {
            if c == 'm' || c == 'K' || c == 'H' || c == 'J' || c == '\\' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += c.width().unwrap_or(0);
        }
    }

    len
}

/// Result of wrapping text.
pub struct WrappedText {
    pub lines: Vec<String>,
}

impl WrappedText {
    pub fn empty() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Split text into words while preserving ANSI codes.
pub fn split_text(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_escape = false;
    let mut escape_buf = String::new();

    for ch in text.chars() {
        if in_escape {
            escape_buf.push(ch);
            if ch == 'm' {
                current.push_str(&escape_buf);
                escape_buf.clear();
                in_escape = false;
            }
            continue;
        }

        if ch == '\x1b' {
            in_escape = true;
            escape_buf.push(ch);
            continue;
        }

        if ch.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if !escape_buf.is_empty() {
        current.push_str(&escape_buf);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Simple text wrap for plain text.
pub fn simple_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let current_len = current.chars().count();

        if current.is_empty() {
            current = word.to_string();
        } else if current_len + 1 + word_len <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Wrap text to fit within a given width (ANSI-aware).
pub fn text_wrap(
    text: &str,
    width: usize,
    first_prefix: &str,
    next_prefix: &str,
) -> WrappedText {
    if width == 0 {
        return WrappedText::empty();
    }

    let words = split_text(text);
    if words.is_empty() {
        return WrappedText::empty();
    }

    let first_prefix_len = visible_length(first_prefix);
    let next_prefix_len = visible_length(next_prefix);

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0;
    let mut is_first_line = true;

    for word in &words {
        let word_len = visible_length(word);
        let prefix_len = if is_first_line {
            first_prefix_len
        } else {
            next_prefix_len
        };
        let available = width.saturating_sub(prefix_len);

        let space_needed = if current_line.is_empty() { 0 } else { 1 };

        if current_len + word_len + space_needed <= available {
            if !current_line.is_empty() {
                current_line.push(' ');
                current_len += 1;
            }
            current_line.push_str(word);
            current_len += word_len;
        } else {
            // Finalize current line
            if !current_line.is_empty() {
                let prefix = if is_first_line {
                    first_prefix
                } else {
                    next_prefix
                };
                lines.push(format!("{}{}", prefix, current_line));
                is_first_line = false;
            }
            // Start new line
            current_line = word.clone();
            current_len = word_len;
        }
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        let prefix = if is_first_line {
            first_prefix
        } else {
            next_prefix
        };
        lines.push(format!("{}{}", prefix, current_line));
    }

    WrappedText { lines }
}
