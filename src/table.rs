//! Table rendering with box-drawing characters.

use crate::text::visible_length;
use crate::theme::Theme;
use streamdown_parser::format_line;

/// Render a buffered table with proper column widths.
pub fn render_table(rows: &[Vec<String>], margin: &str, theme: &Theme) -> Vec<String> {
    if rows.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();

    // Calculate column widths
    let mut widths: Vec<usize> = Vec::new();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let cell_len = visible_length(cell);
            if i >= widths.len() {
                widths.push(cell_len);
            } else {
                widths[i] = widths[i].max(cell_len);
            }
        }
    }

    // Top border
    let top: String = widths
        .iter()
        .map(|&w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┬");
    result.push(format!(
        "{}{}",
        margin,
        theme.table_border.apply(&format!("┌{}┐", top))
    ));

    for (row_idx, row) in rows.iter().enumerate() {
        let is_header = row_idx == 0;
        let cells: String = row
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let w = widths.get(i).copied().unwrap_or(visible_length(c));
                let formatted = format_line(c, true, true);
                let content_len = visible_length(&formatted);
                let padding = w.saturating_sub(content_len);
                if is_header {
                    format!(
                        " {}{}",
                        theme.table_header.apply(&formatted),
                        " ".repeat(padding + 1)
                    )
                } else {
                    format!(" {}{} ", formatted, " ".repeat(padding))
                }
            })
            .collect::<Vec<_>>()
            .join(&theme.table_border.apply("│").to_string());
        result.push(format!(
            "{}{}{}{}",
            margin,
            theme.table_border.apply("│"),
            cells,
            theme.table_border.apply("│")
        ));

        // Separator after header
        if is_header && rows.len() > 1 {
            let sep: String = widths
                .iter()
                .map(|&w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┼");
            result.push(format!(
                "{}{}",
                margin,
                theme.table_border.apply(&format!("├{}┤", sep))
            ));
        }
    }

    // Bottom border
    let bottom: String = widths
        .iter()
        .map(|&w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┴");
    result.push(format!(
        "{}{}",
        margin,
        theme.table_border.apply(&format!("└{}┘", bottom))
    ));

    result
}
