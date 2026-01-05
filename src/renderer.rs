//! Main renderer that handles all parse events.

use std::io::{self, Write};

use streamdown_parser::{decode_html_entities, InlineElement, ParseEvent};

use crate::code::CodeHighlighter;
use crate::heading::render_heading;
use crate::list::{render_list_item, ListState};
use crate::table::render_table;
use crate::text::text_wrap;
use crate::theme::Theme;

/// Main renderer for streamdown events.
pub struct Renderer<W: Write> {
    writer: W,
    width: usize,
    theme: Theme,
    // Code highlighting
    highlighter: CodeHighlighter,
    current_language: Option<String>,
    code_buffer: String,
    // Table buffering
    table_rows: Vec<Vec<String>>,
    // Blockquote state
    in_blockquote: bool,
    blockquote_depth: usize,
    // List state
    list_state: ListState,
    // Column tracking
    column: usize,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W, width: usize) -> Self {
        Self::with_theme(writer, width, Theme::default())
    }

    pub fn with_theme(writer: W, width: usize, theme: Theme) -> Self {
        Self {
            writer,
            width,
            theme,
            highlighter: CodeHighlighter::default(),
            current_language: None,
            code_buffer: String::new(),
            table_rows: Vec::new(),
            in_blockquote: false,
            blockquote_depth: 0,
            list_state: ListState::default(),
            column: 0,
        }
    }

    /// Set a new theme.
    #[allow(dead_code)]
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Get the current theme.
    #[allow(dead_code)]
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Calculate the left margin based on blockquote depth.
    fn left_margin(&self) -> String {
        if self.in_blockquote {
            let border = self.theme.blockquote_border.apply("│").to_string();
            format!("{} ", border).repeat(self.blockquote_depth)
        } else {
            String::new()
        }
    }

    /// Calculate the current available width.
    fn current_width(&self) -> usize {
        let margin_width = if self.in_blockquote {
            self.blockquote_depth * 3
        } else {
            0
        };
        self.width.saturating_sub(margin_width)
    }

    fn write(&mut self, s: &str) -> io::Result<()> {
        write!(self.writer, "{}", s)
    }

    fn writeln(&mut self, s: &str) -> io::Result<()> {
        writeln!(self.writer, "{}", s)?;
        self.column = 0;
        Ok(())
    }

    fn flush_table(&mut self) -> io::Result<()> {
        if self.table_rows.is_empty() {
            return Ok(());
        }
        let rows = std::mem::take(&mut self.table_rows);
        let margin = self.left_margin();
        let lines = render_table(&rows, &margin, &self.theme, self.width);
        for line in lines {
            self.writeln(&line)?;
        }
        Ok(())
    }

    /// Check if this event should reset a pending list.
    /// List continues only for ListItem, ListEnd, and EmptyLine/Newline events.
    fn should_reset_list(event: &ParseEvent) -> bool {
        !matches!(
            event,
            ParseEvent::ListItem { .. }
                | ParseEvent::ListEnd
                | ParseEvent::EmptyLine
                | ParseEvent::Newline
        )
    }

    /// Render a single parse event.
    pub fn render_event(&mut self, event: &ParseEvent) -> io::Result<()> {
        // Reset pending list if this event breaks the list context
        if Self::should_reset_list(event) {
            self.list_state.reset();
        }

        match event {
            // === Inline elements ===
            ParseEvent::Text(text) => {
                let decoded = decode_html_entities(text);
                self.write(&decoded)?;
                self.column += decoded.chars().count();
            }

            ParseEvent::InlineCode(code) => {
                self.write(&self.theme.code.apply(code).to_string())?;
            }

            ParseEvent::Bold(text) => {
                self.write(&self.theme.bold.apply(text).to_string())?;
            }

            ParseEvent::Italic(text) => {
                self.write(&self.theme.italic.apply(text).to_string())?;
            }

            ParseEvent::BoldItalic(text) => {
                let styled = self.theme.bold.apply(text);
                self.write(&self.theme.italic.apply(&styled.to_string()).to_string())?;
            }

            ParseEvent::Underline(text) => {
                self.write(&format!("\x1b[4m{}\x1b[24m", text))?;
            }

            ParseEvent::Strikeout(text) => {
                self.write(&self.theme.strikethrough.apply(text).to_string())?;
            }

            ParseEvent::Link { text, url } => {
                self.write("\x1b]8;;")?;
                self.write(url)?;
                self.write("\x1b\\")?;
                self.write(&self.theme.link.apply(text).to_string())?;
                self.write("\x1b]8;;\x1b\\")?;
                self.write(" ")?;
                self.write(&self.theme.link_url.apply(&format!("({})", url)).to_string())?;
            }

            ParseEvent::Image { alt, url: _ } => {
                self.write(&format!("[🖼 {}]", alt))?;
            }

            ParseEvent::Footnote(superscript) => {
                self.write(superscript)?;
            }

            ParseEvent::Prompt(prompt) => {
                self.write(prompt)?;
            }

            // === Block elements ===
            ParseEvent::Heading { level, content } => {
                let margin = self.left_margin();
                let width = self.current_width();
                let lines = render_heading(*level, content, width, &margin, &self.theme);
                for line in lines {
                    self.writeln(&line)?;
                }
            }

            ParseEvent::CodeBlockStart { language, .. } => {
                self.current_language = language.clone();
                self.code_buffer.clear();
            }

            ParseEvent::CodeBlockLine(line) => {
                if !self.code_buffer.is_empty() {
                    self.code_buffer.push('\n');
                }
                self.code_buffer.push_str(line);

                let margin = self.left_margin();
                let width = self.current_width();
                let rendered_lines = self.highlighter.render_code_line(
                    line,
                    self.current_language.as_deref(),
                    &margin,
                    width,
                );
                for rendered in rendered_lines {
                    self.writeln(&rendered)?;
                }
            }

            ParseEvent::CodeBlockEnd => {
                self.current_language = None;
                self.code_buffer.clear();
            }

            ParseEvent::ListItem {
                indent,
                bullet,
                content,
            } => {
                let margin = self.left_margin();
                let width = self.current_width();
                let lines = render_list_item(
                    *indent,
                    bullet,
                    content,
                    width,
                    &margin,
                    &self.theme,
                    &mut self.list_state,
                );
                for line in lines {
                    self.writeln(&line)?;
                }
            }

            ParseEvent::ListEnd => {
                // Mark as pending - will reset if non-list event arrives
                self.list_state.mark_pending_reset();
            }

            ParseEvent::TableHeader(cols) | ParseEvent::TableRow(cols) => {
                self.table_rows.push(cols.clone());
            }

            ParseEvent::TableSeparator => {}

            ParseEvent::TableEnd => {
                self.flush_table()?;
            }

            ParseEvent::BlockquoteStart { depth } => {
                self.in_blockquote = true;
                self.blockquote_depth = *depth;
            }

            ParseEvent::BlockquoteLine(text) => {
                let margin = self.left_margin();
                let width = self.current_width();
                let wrapped = text_wrap(text, width, &margin, &margin);
                if wrapped.is_empty() {
                    self.writeln(&margin)?;
                } else {
                    for line in wrapped.lines {
                        self.writeln(&line)?;
                    }
                }
            }

            ParseEvent::BlockquoteEnd => {
                self.in_blockquote = false;
                self.blockquote_depth = 0;
            }

            ParseEvent::ThinkBlockStart => {
                self.writeln(&self.theme.think_border.apply("┌─ thinking ─").to_string())?;
                self.in_blockquote = true;
                self.blockquote_depth = 1;
            }

            ParseEvent::ThinkBlockLine(text) => {
                let border = self.theme.think_border.apply("│").to_string();
                self.writeln(&format!("{} {}", border, self.theme.think.apply(text)))?;
            }

            ParseEvent::ThinkBlockEnd => {
                self.writeln(&self.theme.think_border.apply("└").to_string())?;
                self.in_blockquote = false;
                self.blockquote_depth = 0;
            }

            ParseEvent::HorizontalRule => {
                let margin = self.left_margin();
                let rule = "─".repeat(self.current_width());
                self.writeln(&format!("{}{}", margin, self.theme.hr.apply(&rule)))?;
            }

            ParseEvent::EmptyLine | ParseEvent::Newline => {
                self.writeln("")?;
            }

            ParseEvent::InlineElements(elements) => {
                for element in elements {
                    self.render_inline_element(element)?;
                }
            }
        }

        self.writer.flush()
    }

    fn render_inline_element(&mut self, element: &InlineElement) -> io::Result<()> {
        match element {
            InlineElement::Text(s) => self.write(s)?,
            InlineElement::Bold(s) => self.write(&self.theme.bold.apply(s).to_string())?,
            InlineElement::Italic(s) => self.write(&self.theme.italic.apply(s).to_string())?,
            InlineElement::BoldItalic(s) => {
                let styled = self.theme.bold.apply(s);
                self.write(&self.theme.italic.apply(&styled.to_string()).to_string())?;
            }
            InlineElement::Underline(s) => self.write(&format!("\x1b[4m{}\x1b[24m", s))?,
            InlineElement::Strikeout(s) => {
                self.write(&self.theme.strikethrough.apply(s).to_string())?
            }
            InlineElement::Code(s) => self.write(&self.theme.code.apply(s).to_string())?,
            InlineElement::Link { text, url } => {
                self.write("\x1b]8;;")?;
                self.write(url)?;
                self.write("\x1b\\")?;
                self.write(&self.theme.link.apply(text).to_string())?;
                self.write("\x1b]8;;\x1b\\")?;
                self.write(" ")?;
                self.write(&self.theme.link_url.apply(&format!("({})", url)).to_string())?;
            }
            InlineElement::Image { alt, .. } => {
                self.write(&format!("[🖼 {}]", alt))?;
            }
            InlineElement::Footnote(s) => {
                self.write(s)?;
            }
        }
        Ok(())
    }

    /// Consume the renderer and return the inner writer.
    pub fn into_writer(self) -> W {
        self.writer
    }
}
