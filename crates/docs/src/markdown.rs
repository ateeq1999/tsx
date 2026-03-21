//! Minimal Markdown → ratatui styled lines renderer.
//!
//! Supports: headings (#, ##, ###), bold (**), inline code (`), horizontal
//! rules (---), blockquotes (>), bullet lists (- / *), numbered lists.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn render_markdown(content: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in content.lines() {
        lines.push(render_line(line));
    }

    lines
}

fn render_line(line: &str) -> Line<'static> {
    // Headings
    if let Some(rest) = line.strip_prefix("### ") {
        return Line::from(Span::styled(
            format!("  ▸ {}", rest),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(rest) = line.strip_prefix("## ") {
        return Line::from(Span::styled(
            format!("▸ {}", rest),
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(rest) = line.strip_prefix("# ") {
        return Line::from(Span::styled(
            rest.to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
    }

    // Horizontal rule
    if line.trim() == "---" || line.trim() == "***" {
        return Line::from(Span::styled(
            "─".repeat(60),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Blockquote
    if let Some(rest) = line.strip_prefix("> ") {
        return Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(rest.to_string(), Style::default().fg(Color::Gray)),
        ]);
    }

    // Bullet list
    if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        let mut spans = vec![Span::styled("  • ", Style::default().fg(Color::Yellow))];
        spans.extend(render_inline(rest));
        return Line::from(spans);
    }

    // Numbered list: "1. " "2. " etc.
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = " ".repeat(indent_len);
    if trimmed.len() > 2 {
        let chars: Vec<char> = trimmed.chars().collect();
        if chars[0].is_ascii_digit() && chars[1] == '.' && chars[2] == ' ' {
            let num: String = chars.iter().take_while(|c| c.is_ascii_digit()).collect();
            let rest: String = chars[num.len() + 2..].iter().collect();
            let mut spans = vec![Span::styled(
                format!("{}{}. ", indent, num),
                Style::default().fg(Color::Yellow),
            )];
            spans.extend(render_inline(&rest));
            return Line::from(spans);
        }
    }

    // Code block fences (just style differently)
    if line.starts_with("```") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Regular text with inline formatting
    Line::from(render_inline(line))
}

/// Render inline formatting: **bold**, `code`, *italic*
fn render_inline(line: &str) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut current = String::new();

    while i < len {
        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 2;
            let start = i;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            let bold: String = chars[start..i].iter().collect();
            spans.push(Span::styled(bold, Style::default().add_modifier(Modifier::BOLD)));
            i += 2;
            continue;
        }

        // Inline code: `code`
        if chars[i] == '`' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 1;
            let start = i;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            let code: String = chars[start..i].iter().collect();
            spans.push(Span::styled(
                format!("`{}`", code),
                Style::default().fg(Color::Green),
            ));
            if i < len { i += 1; }
            continue;
        }

        // Italic: *text*
        if chars[i] == '*' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 1;
            let start = i;
            while i < len && chars[i] != '*' {
                i += 1;
            }
            let italic: String = chars[start..i].iter().collect();
            spans.push(Span::styled(italic, Style::default().add_modifier(Modifier::ITALIC)));
            if i < len { i += 1; }
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    if spans.is_empty() {
        spans.push(Span::raw(""));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading() {
        let lines = render_markdown("# Hello World");
        assert!(!lines.is_empty());
    }

    #[test]
    fn renders_bullet() {
        let lines = render_markdown("- item one\n- item two");
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn renders_plain_text() {
        let lines = render_markdown("just plain text");
        assert_eq!(lines.len(), 1);
    }
}
