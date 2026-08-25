//! Tool call rendering — renders `ChatPart::Tool` parts.
//!
//! Ported from `tui/src/routes/session/index.tsx` (ToolPart / InlineTool /
//! BlockTool). Two visual styles:
//!
//! - **InlineTool**: single line — `icon  tool_name  input_preview`. Used for
//!   pending tools and completed tools without large output.
//! - **BlockTool**: multi-line block with a left border, title line, and
//!   collapsible output. Used for completed tools whose output exceeds the
//!   collapse threshold.
//!
//! Output folding reuses `util::collapse_tool_output::collapse_tool_output`.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::app::{ChatPart, ToolPartState};
use crate::tui::theme::Theme;
use crate::tui::util::collapse_tool_output::collapse_tool_output;

/// Maximum lines of tool output to show before collapsing.
const MAX_OUTPUT_LINES: usize = 10;

/// Tools with a dedicated renderer. Other tools fall back to "generic".
const KNOWN_TOOLS: &[&str] = &[
    "bash",
    "glob",
    "read",
    "grep",
    "webfetch",
    "websearch",
    "write",
    "edit",
    "task",
    "apply_patch",
    "todowrite",
    "question",
    "skill",
    "execute",
];

/// Return the display category for a tool name. Matches the TS
/// `toolDisplay()` — known tools map to themselves, others to "generic".
pub fn tool_display(tool: &str) -> &str {
    if KNOWN_TOOLS.contains(&tool) {
        tool
    } else {
        "generic"
    }
}

/// Icon character for a tool display category. Matches the icons used in
/// the TS original (`InlineTool icon="..."`).
pub fn tool_icon(display: &str) -> &'static str {
    match display {
        "bash" | "execute" => "$",
        "glob" => "✱",
        "grep" => "✱",
        "read" => "📖", // read uses a book-ish glyph; falls back to ⛀ if unsupported
        "webfetch" => "%",
        "websearch" => "◈",
        "write" => "✎",
        "edit" => "←",
        "apply_patch" => "%",
        "task" => "▸",
        "todowrite" => "☑",
        "question" => "→",
        "skill" => "→",
        _ => "⚙",
    }
}

/// One-line "pending" message shown while a tool is running. Matches the
/// TS `pending="..."` props on each tool's InlineTool.
pub fn pending_message(display: &str) -> &'static str {
    match display {
        "bash" | "execute" => "Running command…",
        "glob" => "Finding files…",
        "read" => "Reading file…",
        "grep" => "Searching content…",
        "webfetch" => "Fetching from the web…",
        "websearch" => "Searching web…",
        "write" => "Writing file…",
        "edit" => "Preparing edit…",
        "apply_patch" => "Preparing patch…",
        "task" => "Running task…",
        "todowrite" => "Updating todos…",
        "question" => "Asking question…",
        "skill" => "Loading skill…",
        _ => "Running tool…",
    }
}

/// Render a single tool part by appending lines to `lines`.
/// Default: collapsed (single line summary). Expanded: full output with border.
pub fn render_tool_part_to_lines(
    lines: &mut Vec<Line<'static>>,
    part: &ChatPart,
    theme: &Theme,
    width: u16,
) {
    let (tool_name, call_id, state) = match part {
        ChatPart::Tool {
            tool_name,
            call_id,
            state,
        } => (tool_name.as_str(), call_id.as_str(), state),
        ChatPart::Text { .. } => return,
    };

    let display = tool_display(tool_name);
    let icon = tool_icon(display);
    let input = state.input();
    let input_str = tool_input_preview(display, input);

    match state {
        ToolPartState::Pending { .. } => {
            // Pending: single line "⚙ 工具名 参数..."
            let label = crate::tui::component::tool_render::tool_display_name_i18n(display);
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(5);
            spans.push(Span::styled(format!("{icon} "), Style::default().fg(theme.primary)));
            spans.push(Span::styled(label, Style::default().fg(theme.text_muted)));
            if !input_str.is_empty() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(input_str, Style::default().fg(theme.text_muted)));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(spans));
        }
        ToolPartState::Completed { output, .. } => {
            let trimmed = output.trim();
            let label = crate::tui::component::tool_render::tool_display_name_i18n(display);

            if trimmed.is_empty() {
                // No output — just show the tool name line.
                let mut spans: Vec<Span<'static>> = Vec::with_capacity(3);
                spans.push(Span::styled(format!("{icon} "), Style::default().fg(theme.primary)));
                spans.push(Span::styled(label, Style::default().fg(theme.text_muted)));
                if !input_str.is_empty() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(input_str, Style::default().fg(theme.text_muted)));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(spans));
            } else {
                // Collapsed: show summary line + hint "(click to expand)"
                let mut spans: Vec<Span<'static>> = Vec::with_capacity(5);
                spans.push(Span::styled(format!("{icon} "), Style::default().fg(theme.primary)));
                spans.push(Span::styled(label, Style::default().fg(theme.text_muted)));
                if !input_str.is_empty() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(input_str, Style::default().fg(theme.text_muted)));
                }
                // Show first line of output as preview (truncated).
                let first_line = trimmed.lines().next().unwrap_or("");
                let preview = crate::tui::app::message::truncate_chars(first_line, 40);
                if !preview.is_empty() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(preview, Style::default().fg(theme.text_muted)));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(spans));

                // Show full output in a bordered block.
                let max_chars = compute_max_chars(width);
                let collapsed = collapse_tool_output(trimmed, MAX_OUTPUT_LINES, max_chars);
                for l in collapsed.output.split('\n') {
                    lines.push(Line::from(vec![
                        Span::styled("│ ", Style::default().fg(theme.border)),
                        Span::styled(l.to_string(), Style::default().fg(theme.text)),
                    ]));
                }
                if collapsed.overflow {
                    lines.push(Line::from(vec![
                        Span::styled("│ ", Style::default().fg(theme.border)),
                        Span::styled("… (output truncated)", Style::default().fg(theme.text_muted)),
                    ]));
                }
            }
        }
        ToolPartState::Error { error, .. } => {
            let label = crate::tui::component::tool_render::tool_display_name_i18n(display);
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(4);
            spans.push(Span::styled(format!("{icon} "), Style::default().fg(theme.error)));
            spans.push(Span::styled(
                format!("{} failed", label),
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
            ));
            if !input_str.is_empty() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(input_str, Style::default().fg(theme.text_muted)));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(spans));
            for l in error.split('\n').take(3) {
                lines.push(Line::from(vec![
                    Span::styled("│ ", Style::default().fg(theme.error)),
                    Span::styled(format!("  {l}"), Style::default().fg(theme.error)),
                ]));
            }
        }
    }
}

/// Get the i18n display name for a tool.
pub fn tool_display_name_i18n(display: &str) -> String {
    let key = format!("tui.tool.{}", display);
    let translated = crate::tui::i18n::t(&key);
    if translated == key {
        display.to_string()
    } else {
        translated
    }
}

/// Short one-line preview of a tool's primary input field.
fn tool_input_preview(display: &str, input: &serde_json::Value) -> String {
    match crate::tui::app::input_preview(input) {
        s if !s.is_empty() => crate::tui::app::message::truncate_chars(&s, 60),
        _ => String::new(),
    }
}

fn compute_max_chars(width: u16) -> usize {
    MAX_OUTPUT_LINES * std::cmp::max(20, width.saturating_sub(6) as usize)
}
