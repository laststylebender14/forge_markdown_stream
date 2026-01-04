//! List rendering with nested indentation and bullet cycling.

use crate::inline::render_inline_content;
use crate::text::{text_wrap, visible_length};
use crate::theme::Theme;
use streamdown_parser::ListBullet;

/// Bullet characters for different nesting levels.
const BULLETS: [&str; 4] = ["•", "◦", "▪", "‣"];

/// List rendering state for tracking nesting and numbering.
#[derive(Default)]
pub struct ListState {
    /// Stack of (indent, is_ordered) for nested lists
    stack: Vec<(usize, bool)>,
    /// Current ordered list numbers at each level
    numbers: Vec<usize>,
    /// Whether we're in a "pending" state (saw ListEnd but might continue)
    pending_reset: bool,
}

impl ListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, indent: usize, ordered: bool) {
        self.stack.push((indent, ordered));
        self.numbers.push(0);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
        self.numbers.pop();
    }

    pub fn next_number(&mut self) -> usize {
        if let Some(n) = self.numbers.last_mut() {
            *n += 1;
            *n
        } else {
            1
        }
    }

    pub fn adjust_for_indent(&mut self, indent: usize, ordered: bool) {
        // Pop levels that are deeper than current
        while let Some((stack_indent, _)) = self.stack.last() {
            if *stack_indent > indent {
                self.pop();
            } else {
                break;
            }
        }

        // Check if we need to push a new level
        let need_push = self.stack.last().map(|(i, _)| indent > *i).unwrap_or(true);
        if need_push {
            self.push(indent, ordered);
        }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.numbers.clear();
        self.pending_reset = false;
    }

    /// Mark list as pending reset (saw ListEnd, but might continue with more items)
    pub fn mark_pending_reset(&mut self) {
        self.pending_reset = true;
    }

    /// Resume list if it was pending reset (new list item arrived)
    fn resume_if_pending(&mut self) {
        self.pending_reset = false;
    }
}

/// Render a list item.
pub fn render_list_item(
    indent: usize,
    bullet: &ListBullet,
    content: &str,
    width: usize,
    margin: &str,
    theme: &Theme,
    list_state: &mut ListState,
) -> Vec<String> {
    // Resume list if it was pending reset (continues after empty line)
    list_state.resume_if_pending();

    // Adjust list state for current indent
    let ordered = matches!(bullet, ListBullet::Ordered(_));
    list_state.adjust_for_indent(indent, ordered);

    let level = list_state.level().saturating_sub(1);

    // Calculate marker - use our own counter for ordered lists to work around
    // the parser bug that normalizes all numbers to 1
    let marker = match bullet {
        ListBullet::Ordered(_) => {
            let num = list_state.next_number();
            format!("{}.", num)
        }
        ListBullet::PlusExpand => "⊞".to_string(),
        _ => BULLETS[level % BULLETS.len()].to_string(),
    };

    // Calculate indentation
    let indent_spaces = indent * 2;
    let marker_width = visible_length(&marker);
    let content_indent = indent_spaces + marker_width + 1;

    // Color the marker
    let colored_marker = if matches!(bullet, ListBullet::Ordered(_)) {
        theme.list_number.apply(&marker).to_string()
    } else {
        theme.bullet.apply(&marker).to_string()
    };

    // Parse and render inline content
    let rendered_content = render_inline_content(content, theme);

    // Build prefixes
    let first_prefix = format!("{}{}{} ", margin, " ".repeat(indent_spaces), colored_marker);
    let next_prefix = format!("{}{}", margin, " ".repeat(content_indent));

    // Wrap the content
    let wrapped = text_wrap(&rendered_content, width, &first_prefix, &next_prefix);

    if wrapped.is_empty() {
        vec![first_prefix]
    } else {
        wrapped.lines
    }
}
