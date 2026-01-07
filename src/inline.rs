//! Inline content rendering with theme-based formatting.

use crate::theme::Theme;
use streamdown_parser::{decode_html_entities, InlineElement, InlineParser};

/// Render inline elements to a string using the theme.
pub fn render_inline_content(content: &str, theme: &Theme) -> String {
    let mut parser = InlineParser::new();
    let elements = parser.parse(content);

    let mut result = String::new();

    for element in elements {
        match element {
            InlineElement::Text(text) => {
                result.push_str(&decode_html_entities(&text));
            }
            InlineElement::Bold(text) => {
                result.push_str(&theme.bold.apply(&decode_html_entities(&text)).to_string());
            }
            InlineElement::Italic(text) => {
                result.push_str(&theme.italic.apply(&decode_html_entities(&text)).to_string());
            }
            InlineElement::BoldItalic(text) => {
                // Combine bold and italic
                let decoded = decode_html_entities(&text);
                let styled = theme.bold.apply(&decoded);
                result.push_str(&theme.italic.apply(&styled.to_string()).to_string());
            }
            InlineElement::Strikeout(text) => {
                result.push_str(&theme.strikethrough.apply(&decode_html_entities(&text)).to_string());
            }
            InlineElement::Underline(text) => {
                // Use bold with underline effect
                let decoded = decode_html_entities(&text);
                result.push_str(&format!("\x1b[4m{}\x1b[24m", decoded));
            }
            InlineElement::Code(text) => {
                result.push_str(&theme.code.apply(&text).to_string());
            }
            InlineElement::Link { text, url } => {
                // OSC 8 hyperlink
                result.push_str("\x1b]8;;");
                result.push_str(&url);
                result.push_str("\x1b\\");
                result.push_str(&theme.link.apply(&decode_html_entities(&text)).to_string());
                result.push_str("\x1b]8;;\x1b\\");
                result.push(' ');
                result.push_str(&theme.link_url.apply(&format!("({})", url)).to_string());
            }
            InlineElement::Image { alt, .. } => {
                result.push_str(&format!("[🖼 {}]", alt));
            }
            InlineElement::Footnote(text) => {
                result.push_str(&text);
            }
        }
    }

    result
}



#[cfg(test)]
mod tests {
    use super::*;

    /// Strip ANSI escape codes from a string for easier testing.
    fn strip_ansi(s: &str) -> String {
        String::from_utf8(strip_ansi_escapes::strip(s)).unwrap()
    }

    #[test]
    fn test_plain_text() {
        let theme = Theme::default();
        let result = render_inline_content("hello world", &theme);
        assert_eq!(strip_ansi(&result), "hello world");
    }

    #[test]
    fn test_plain_text_with_html_entities() {
        let theme = Theme::default();
        let result = render_inline_content("&amp; &lt; &gt; &quot;", &theme);
        assert_eq!(strip_ansi(&result), "& < > \"");
    }

    #[test]
    fn test_bold_text() {
        let theme = Theme::default();
        let result = render_inline_content("**bold**", &theme);
        assert_eq!(strip_ansi(&result), "bold");
    }

    #[test]
    fn test_italic_text() {
        let theme = Theme::default();
        let result = render_inline_content("*italic*", &theme);
        assert_eq!(strip_ansi(&result), "italic");
    }

    #[test]
    fn test_bold_italic_text() {
        let theme = Theme::default();
        let result = render_inline_content("***bold italic***", &theme);
        assert_eq!(strip_ansi(&result), "bold italic");
    }

    #[test]
    fn test_strikethrough_text() {
        let theme = Theme::default();
        let result = render_inline_content("~~strikethrough~~", &theme);
        assert_eq!(strip_ansi(&result), "strikethrough");
    }

    #[test]
    fn test_inline_code() {
        let theme = Theme::default();
        let result = render_inline_content("`code`", &theme);
        assert_eq!(strip_ansi(&result), "code");
    }

    #[test]
    fn test_underline_text() {
        let theme = Theme::default();
        let result = render_inline_content("__underline__", &theme);
        assert_eq!(strip_ansi(&result), "underline");
    }

    #[test]
    fn test_link() {
        let theme = Theme::default();
        let result = render_inline_content("[text](https://example.com)", &theme);
        let stripped = strip_ansi(&result);
        assert!(stripped.contains("text"));
        assert!(stripped.contains("https://example.com"));
    }

    #[test]
    fn test_image() {
        let theme = Theme::default();
        let result = render_inline_content("![alt text](image.png)", &theme);
        assert_eq!(strip_ansi(&result), "[🖼 alt text]");
    }

    #[test]
    fn test_mixed_inline_elements() {
        let theme = Theme::default();
        let result = render_inline_content("hello **bold** and *italic*", &theme);
        assert_eq!(strip_ansi(&result), "hello bold and italic");
    }

    #[test]
    fn test_empty_content() {
        let theme = Theme::default();
        let result = render_inline_content("", &theme);
        assert_eq!(strip_ansi(&result), "");
    }

    #[test]
    fn test_html_entities_in_bold() {
        let theme = Theme::default();
        let result = render_inline_content("**&amp;**", &theme);
        assert_eq!(strip_ansi(&result), "&");
    }

    #[test]
    fn test_html_entities_in_link() {
        let theme = Theme::default();
        let result = render_inline_content("[&lt;click&gt;](https://example.com)", &theme);
        assert!(strip_ansi(&result).contains("<click>"));
    }

    #[test]
    fn test_code_preserves_content() {
        let theme = Theme::default();
        let result = render_inline_content("`let x = 1;`", &theme);
        assert_eq!(strip_ansi(&result), "let x = 1;");
    }

    #[test]
    fn test_multiple_bold_segments() {
        let theme = Theme::default();
        let result = render_inline_content("**one** and **two**", &theme);
        assert_eq!(strip_ansi(&result), "one and two");
    }

    #[test]
    fn test_nested_formatting_text() {
        let theme = Theme::default();
        let result = render_inline_content("**bold *nested* bold**", &theme);
        assert!(strip_ansi(&result).contains("bold"));
    }

    #[test]
    fn test_with_default_theme_produces_ansi() {
        let theme = Theme::default();
        let result = render_inline_content("**bold**", &theme);
        // Default theme applies bold styling, which includes ANSI codes
        assert!(result.contains("\x1b["));
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_link_with_special_characters_in_url() {
        let theme = Theme::default();
        let result = render_inline_content("[link](https://example.com/path?q=1&b=2)", &theme);
        assert!(strip_ansi(&result).contains("https://example.com/path?q=1&b=2"));
    }

    #[test]
    fn test_image_with_empty_alt() {
        let theme = Theme::default();
        let result = render_inline_content("![](image.png)", &theme);
        assert_eq!(strip_ansi(&result), "[🖼 ]");
    }

    #[test]
    fn test_whitespace_preservation() {
        let theme = Theme::default();
        let result = render_inline_content("hello   world", &theme);
        assert_eq!(strip_ansi(&result), "hello   world");
    }

    #[test]
    fn test_special_characters() {
        let theme = Theme::default();
        let result = render_inline_content("hello © world ™", &theme);
        assert_eq!(strip_ansi(&result), "hello © world ™");
    }
}
