//! Logo component — renders ASCII art logo.
//! Ported from tui/src/logo.ts and tui/src/component/logo.tsx
//!
//! Left half: textMuted, not bold.
//! Right half: primary, bold.
//! Special characters in the logo data are rendered as block elements:
//!   `_` → space with fg+shadow bg
//!   `^` → ▀ with fg+shadow bg
//!   `~` → ▀ with shadow fg
//!   `,` → ▄ with shadow fg

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::tui::theme::Theme;

const LOGO_LEFT: &[&str] = &[
    "                   ",
    "█▀▀█ █▀▀█ █▀▀█ █▀▀▄",
    "█__█ █__█ █^^^ █__█",
    "▀▀▀▀ █▀▀▀ ▀▀▀▀ ▀~~▀",
];

const LOGO_RIGHT: &[&str] = &[
    "             ▄     ",
    "█▀▀▀ █▀▀█ █▀▀█ █▀▀█",
    "█___ █__█ █__█ █^^^",
    "▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀",
];

fn blend(bg: Color, fg: Color, alpha: f32) -> Color {
    match (bg, fg) {
        (Color::Rgb(br, bg_, bb), Color::Rgb(fr, fg_, fb)) => {
            let r = (br as f32 * (1.0 - alpha) + fr as f32 * alpha) as u8;
            let g = (bg_ as f32 * (1.0 - alpha) + fg_ as f32 * alpha) as u8;
            let b = (bb as f32 * (1.0 - alpha) + fb as f32 * alpha) as u8;
            Color::Rgb(r, g, b)
        }
        _ => fg,
    }
}

fn render_line(line: &str, fg: Color, bg: Color, bold: bool) -> Vec<Span<'_>> {
    let shadow = blend(bg, fg, 0.25);
    let modifier = if bold { Modifier::BOLD } else { Modifier::empty() };

    line.chars()
        .map(|ch| match ch {
            '_' => Span::styled(
                " ",
                Style::default().fg(fg).bg(shadow).add_modifier(modifier),
            ),
            '^' => Span::styled(
                "▀",
                Style::default().fg(fg).bg(shadow).add_modifier(modifier),
            ),
            '~' => Span::styled(
                "▀",
                Style::default().fg(shadow).add_modifier(modifier),
            ),
            ',' => Span::styled(
                "▄",
                Style::default().fg(shadow).add_modifier(modifier),
            ),
            _ => Span::styled(ch.to_string(), Style::default().fg(fg).add_modifier(modifier)),
        })
        .collect()
}

pub fn render_logo(area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, theme: &Theme) {
    let mut lines: Vec<Line> = Vec::with_capacity(LOGO_LEFT.len());

    for (i, left_line) in LOGO_LEFT.iter().enumerate() {
        let right_line = LOGO_RIGHT.get(i).unwrap_or(&"");
        let bg = theme.background;

        let mut spans = render_line(left_line, theme.text_muted, bg, false);
        spans.push(Span::raw(" "));
        spans.extend(render_line(right_line, theme.primary, bg, true));

        lines.push(Line::from(spans));
    }

    let paragraph = ratatui::widgets::Paragraph::new(lines);
    paragraph.render(area, buf);
}
