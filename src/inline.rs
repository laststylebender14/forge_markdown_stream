//! Inline content rendering with theme-based formatting.

use crate::theme::Theme;
use streamdown_parser::{decode_html_entities, InlineElement, InlineParser};

/// Trait for styling inline elements.
pub trait InlineStyler {
    fn text(&self, text: &str) -> String;
    fn bold(&self, text: &str) -> String;
    fn italic(&self, text: &str) -> String;
    fn bold_italic(&self, text: &str) -> String;
    fn strikethrough(&self, text: &str) -> String;
    fn underline(&self, text: &str) -> String;
    fn code(&self, text: &str) -> String;
    fn link(&self, text: &str, url: &str) -> String;
    fn image(&self, alt: &str, url: &str) -> String;
    fn footnote(&self, text: &str) -> String;
}

/// Theme-based styler that outputs ANSI codes.
impl InlineStyler for Theme {
    fn text(&self, text: &str) -> String {
        decode_html_entities(text)
    }

    fn bold(&self, text: &str) -> String {
        self.bold.apply(&decode_html_entities(text)).to_string()
    }

    fn italic(&self, text: &str) -> String {
        self.italic.apply(&decode_html_entities(text)).to_string()
    }

    fn bold_italic(&self, text: &str) -> String {
        let decoded = decode_html_entities(text);
        let styled = self.bold.apply(&decoded);
        self.italic.apply(&styled.to_string()).to_string()
    }

    fn strikethrough(&self, text: &str) -> String {
        self.strikethrough.apply(&decode_html_entities(text)).to_string()
    }

    fn underline(&self, text: &str) -> String {
        format!("\x1b[4m{}\x1b[24m", decode_html_entities(text))
    }

    fn code(&self, text: &str) -> String {
        self.code.apply(text).to_string()
    }

    fn link(&self, text: &str, url: &str) -> String {
        let mut result = String::new();
        result.push_str("\x1b]8;;");
        result.push_str(url);
        result.push_str("\x1b\\");
        result.push_str(&self.link.apply(&decode_html_entities(text)).to_string());
        result.push_str("\x1b]8;;\x1b\\");
        result.push(' ');
        result.push_str(&self.link_url.apply(&format!("({})", url)).to_string());
        result
    }

    fn image(&self, alt: &str, _url: &str) -> String {
        format!("[🖼 {}]", alt)
    }

    fn footnote(&self, text: &str) -> String {
        text.to_string()
    }
}

/// Render inline elements to a string using a styler.
pub fn render_inline_content<S: InlineStyler>(content: &str, styler: &S) -> String {
    let mut parser = InlineParser::new();
    let elements = parser.parse(content);

    let mut result = String::new();

    for element in elements {
        match element {
            InlineElement::Text(text) => {
                result.push_str(&styler.text(&text));
            }
            InlineElement::Bold(text) => {
                result.push_str(&styler.bold(&text));
            }
            InlineElement::Italic(text) => {
                result.push_str(&styler.italic(&text));
            }
            InlineElement::BoldItalic(text) => {
                result.push_str(&styler.bold_italic(&text));
            }
            InlineElement::Strikeout(text) => {
                result.push_str(&styler.strikethrough(&text));
            }
            InlineElement::Underline(text) => {
                result.push_str(&styler.underline(&text));
            }
            InlineElement::Code(text) => {
                result.push_str(&styler.code(&text));
            }
            InlineElement::Link { text, url } => {
                result.push_str(&styler.link(&text, &url));
            }
            InlineElement::Image { alt, url } => {
                result.push_str(&styler.image(&alt, &url));
            }
            InlineElement::Footnote(text) => {
                result.push_str(&styler.footnote(&text));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test styler that outputs readable HTML-like tags.
    struct TagStyler;

    impl InlineStyler for TagStyler {
        fn text(&self, text: &str) -> String {
            decode_html_entities(text)
        }

        fn bold(&self, text: &str) -> String {
            format!("<b>{}</b>", decode_html_entities(text))
        }

        fn italic(&self, text: &str) -> String {
            format!("<i>{}</i>", decode_html_entities(text))
        }

        fn bold_italic(&self, text: &str) -> String {
            format!("<b><i>{}</i></b>", decode_html_entities(text))
        }

        fn strikethrough(&self, text: &str) -> String {
            format!("<s>{}</s>", decode_html_entities(text))
        }

        fn underline(&self, text: &str) -> String {
            format!("<u>{}</u>", decode_html_entities(text))
        }

        fn code(&self, text: &str) -> String {
            format!("<code>{}</code>", text)
        }

        fn link(&self, text: &str, url: &str) -> String {
            format!("<a href=\"{}\">{}</a>", url, decode_html_entities(text))
        }

        fn image(&self, alt: &str, url: &str) -> String {
            format!("<img alt=\"{}\" src=\"{}\"/>", alt, url)
        }

        fn footnote(&self, text: &str) -> String {
            format!("<footnote>{}</footnote>", text)
        }
    }

    fn render(content: &str) -> String {
        render_inline_content(content, &TagStyler)
    }

    #[test]
    fn test_plain_text() {
        insta::assert_snapshot!(render("hello world"), @"hello world");
    }

    #[test]
    fn test_html_entities() {
        insta::assert_snapshot!(render("&amp; &lt; &gt; &quot;"), @r#"& < > ""#);
    }

    #[test]
    fn test_bold() {
        insta::assert_snapshot!(render("**bold**"), @"<b>bold</b>");
    }

    #[test]
    fn test_italic() {
        insta::assert_snapshot!(render("*italic*"), @"<i>italic</i>");
    }

    #[test]
    fn test_bold_italic() {
        insta::assert_snapshot!(render("***text***"), @"<b><i>text</i></b>");
    }

    #[test]
    fn test_strikethrough() {
        insta::assert_snapshot!(render("~~struck~~"), @"<s>struck</s>");
    }

    #[test]
    fn test_code() {
        insta::assert_snapshot!(render("`code`"), @"<code>code</code>");
    }

    #[test]
    fn test_underline() {
        insta::assert_snapshot!(render("__underline__"), @"<u>underline</u>");
    }

    #[test]
    fn test_link() {
        insta::assert_snapshot!(render("[click](https://example.com)"), @r#"<a href="https://example.com">click</a>"#);
    }

    #[test]
    fn test_image() {
        insta::assert_snapshot!(render("![alt](image.png)"), @r#"<img alt="alt" src="image.png"/>"#);
    }

    #[test]
    fn test_mixed() {
        insta::assert_snapshot!(render("hello **bold** and *italic*"), @"hello <b>bold</b> and <i>italic</i>");
    }

    #[test]
    fn test_multiple_bold() {
        insta::assert_snapshot!(render("**one** and **two**"), @"<b>one</b> and <b>two</b>");
    }

    #[test]
    fn test_entities_in_bold() {
        insta::assert_snapshot!(render("**&amp;**"), @"<b>&</b>");
    }

    #[test]
    fn test_entities_in_link() {
        insta::assert_snapshot!(render("[&lt;click&gt;](https://example.com)"), @r#"<a href="https://example.com"><click></a>"#);
    }

    #[test]
    fn test_code_content() {
        insta::assert_snapshot!(render("`let x = 1;`"), @"<code>let x = 1;</code>");
    }

    #[test]
    fn test_link_special_url() {
        insta::assert_snapshot!(render("[link](https://example.com/path?q=1&b=2)"), @r#"<a href="https://example.com/path?q=1&b=2">link</a>"#);
    }

    #[test]
    fn test_empty() {
        insta::assert_snapshot!(render(""), @"");
    }

    #[test]
    fn test_whitespace() {
        insta::assert_snapshot!(render("hello   world"), @"hello   world");
    }

    #[test]
    fn test_image_empty_alt() {
        insta::assert_snapshot!(render("![](image.png)"), @r#"<img alt="" src="image.png"/>"#);
    }

    // Verify Theme implementation produces ANSI
    #[test]
    fn test_theme_produces_ansi() {
        let theme = Theme::default();
        let result = render_inline_content("**bold**", &theme);
        assert!(result.contains("\x1b["), "Expected ANSI codes from Theme");
    }
}
