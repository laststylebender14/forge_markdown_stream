//! Table rendering with box-drawing characters.

use crate::text::visible_length;
use crate::theme::Theme;
use streamdown_parser::format_line;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Render a table with proper column widths, shrinking and wrapping if needed.
pub fn render_table(rows: &[Vec<String>], margin: &str, theme: &Theme, max_width: usize) -> Vec<String> {
    let n = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if rows.is_empty() || n == 0 { return vec![]; }

    let mut w: Vec<usize> = vec![0; n];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            w[i] = w[i].max(visible_length(&format_line(cell, true, true)));
        }
    }

    let overhead = margin.width() + 1 + 3 * n;
    let total: usize = w.iter().sum();
    if overhead + total > max_width && max_width > overhead {
        let avail = max_width - overhead;
        w.iter_mut().for_each(|x| *x = (*x * avail / total).max(5));
    }

    let b = &theme.table_border;
    let hline = |l, m, r| format!("{}{}{}{}", margin, b.apply(l),
        w.iter().map(|&x| b.apply(&"─".repeat(x + 2)).to_string()).collect::<Vec<_>>().join(&b.apply(m).to_string()), b.apply(r));

    let mut out = vec![hline("┌", "┬", "┐")];
    for (ri, row) in rows.iter().enumerate() {
        let wrapped: Vec<Vec<String>> = (0..n)
            .map(|i| wrap(&format_line(row.get(i).map(|s| s.as_str()).unwrap_or(""), true, true), w[i]))
            .collect();
        for li in 0..wrapped.iter().map(|c| c.len()).max().unwrap_or(1) {
            let cells: String = (0..n).map(|i| {
                let c = wrapped[i].get(li).map(|s| s.as_str()).unwrap_or("");
                let p = " ".repeat(w[i].saturating_sub(visible_length(c)));
                if ri == 0 && li == 0 && !c.is_empty() { format!(" {}{} ", theme.table_header.apply(c), p) }
                else { format!(" {}{} ", c, p) }
            }).collect::<Vec<_>>().join(&b.apply("│").to_string());
            out.push(format!("{}{}{}{}", margin, b.apply("│"), cells, b.apply("│")));
        }
        if ri < rows.len() - 1 { out.push(hline("├", "┼", "┤")); }
    }
    out.push(hline("└", "┴", "┘"));
    out
}

/// Wrap text by characters, preserving ANSI codes across lines.
fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 || visible_length(text) <= width { return vec![text.to_string()]; }

    let (mut lines, mut line, mut w, mut esc, mut style) = (vec![], String::new(), 0, String::new(), None::<String>);
    for c in text.chars() {
        if !esc.is_empty() || c == '\x1b' {
            esc.push(c);
            if c == 'm' { style = if esc == "\x1b[0m" { None } else { Some(esc.clone()) }; line.push_str(&esc); esc.clear(); }
        } else {
            let cw = c.width().unwrap_or(0);
            if w + cw > width && w > 0 { line.push_str("\x1b[0m"); lines.push(line); line = style.clone().unwrap_or_default(); w = 0; }
            line.push(c); w += cw;
        }
    }
    if !line.is_empty() { lines.push(line); }
    if lines.is_empty() { vec![String::new()] } else { lines }
}
