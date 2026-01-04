//! Code block rendering with syntax highlighting and line wrapping.

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

const RESET: &str = "\x1b[0m";

/// Code block highlighter using syntect.
pub struct CodeHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for CodeHighlighter {
    fn default() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl CodeHighlighter {
    /// Highlight a single line of code.
    pub fn highlight_line(&self, line: &str, language: Option<&str>) -> String {
        let syntax = language
            .and_then(|lang| self.syntax_set.find_syntax_by_token(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        match highlighter.highlight_line(line, &self.syntax_set) {
            Ok(ranges) => as_24_bit_terminal_escaped(&ranges[..], false),
            Err(_) => line.to_string(),
        }
    }

    /// Render a code line with margin, wrapping if needed.
    ///
    /// Returns multiple lines if the code exceeds the available width.
    pub fn render_code_line(
        &self,
        line: &str,
        language: Option<&str>,
        margin: &str,
        width: usize,
    ) -> Vec<String> {
        let (indent, wrapped_lines) = code_wrap(line, width);

        let mut result = Vec::new();

        for (i, code_line) in wrapped_lines.iter().enumerate() {
            let highlighted = self.highlight_line(code_line, language);

            // Add continuation indent for wrapped lines
            let line_indent = if i == 0 {
                ""
            } else {
                &"  ".repeat(indent.min(4) / 2 + 1)
            };

            result.push(format!("{}{}{}{}", margin, line_indent, highlighted, RESET));
        }

        if result.is_empty() {
            result.push(format!("{}{}", margin, RESET));
        }

        result
    }
}

/// Wrap a code line if it exceeds the width.
///
/// Unlike text wrapping, code wrapping preserves indentation
/// and doesn't break on word boundaries - it breaks at character boundaries.
///
/// # Arguments
/// * `text` - The code line
/// * `width` - Maximum width
///
/// # Returns
/// (indent, lines) - The detected indent level and wrapped lines
pub fn code_wrap(text: &str, width: usize) -> (usize, Vec<String>) {
    if text.is_empty() {
        return (0, vec![String::new()]);
    }

    // Detect indentation
    let indent = text.len() - text.trim_start().len();
    let content = text.trim_start();

    if content.is_empty() {
        return (indent, vec![text.to_string()]);
    }

    // Calculate effective width (accounting for indent on continuation lines)
    // Reserve 4 chars for continuation indent marker
    let effective_width = width.saturating_sub(4).saturating_sub(indent);

    if effective_width == 0 || content.len() <= effective_width {
        return (indent, vec![text.to_string()]);
    }

    // Wrap the content at character boundaries
    let mut lines = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + effective_width).min(chars.len());
        let line_chars: String = chars[start..end].iter().collect();

        if start == 0 {
            // First line includes original indentation
            lines.push(format!("{}{}", " ".repeat(indent), line_chars));
        } else {
            lines.push(line_chars);
        }

        start = end;
    }

    // Remove trailing empty lines
    while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }

    if lines.is_empty() {
        lines.push(text.to_string());
    }

    (indent, lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_wrap_short_line() {
        let (indent, lines) = code_wrap("let x = 1;", 80);
        assert_eq!(indent, 0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "let x = 1;");
    }

    #[test]
    fn test_code_wrap_with_indent() {
        let (indent, lines) = code_wrap("    let x = 1;", 80);
        assert_eq!(indent, 4);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_code_wrap_long_line() {
        let long_line = "x".repeat(100);
        let (_, lines) = code_wrap(&long_line, 40);
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_code_wrap_empty() {
        let (indent, lines) = code_wrap("", 80);
        assert_eq!(indent, 0);
        assert_eq!(lines.len(), 1);
    }
}
