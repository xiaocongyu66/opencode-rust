use chrono::{DateTime, Utc};

pub struct SessionInfo {
    pub title: String,
    pub session_id: Option<String>,
}

pub struct SessionEpilogueInput {
    pub title: String,
    pub session_id: Option<String>,
}

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

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[90m";

fn draw_char(ch: char, fg: &str, shadow: &str, bg: &str) -> String {
    match ch {
        '_' => format!("{} {}", bg, RESET),
        '^' => format!("{}{}▀{}", fg, bg, RESET),
        '~' => format!("{}▀{}", shadow, RESET),
        ' ' => " ".to_string(),
        _ => format!("{}{}{}", fg, ch, RESET),
    }
}

fn draw_line(line: &str, fg: &str, shadow: &str, bg: &str) -> String {
    line.chars().map(|c| draw_char(c, fg, shadow, bg)).collect()
}

fn wordmark(pad: &str) -> Vec<String> {
    LOGO_LEFT
        .iter()
        .enumerate()
        .map(|(i, left_line)| {
            let left = draw_line(left_line, DIM, "\x1b[38;5;235m", "\x1b[48;5;235m");
            let right_line = LOGO_RIGHT.get(i).unwrap_or(&"");
            let right = draw_line(right_line, RESET, "\x1b[38;5;238m", "\x1b[48;5;238m");
            format!("{}{} {}", pad, left, right)
        })
        .collect()
}

fn weak(text: &str) -> String {
    let padded = format!("{:<10}", text);
    format!("{}{}{}", DIM, padded, RESET)
}

pub fn session_epilogue(input: &SessionEpilogueInput) -> String {
    let mut lines: Vec<String> = wordmark("  ");
    lines.push(String::new());
    lines.push(format!(
        "  {}{}{}{}",
        weak("Session"),
        BOLD,
        input.title,
        RESET
    ));
    lines.push(format!(
        "  {}{}opencode -s {}{}",
        weak("Continue"),
        BOLD,
        input.session_id.as_deref().unwrap_or(""),
        RESET
    ));
    lines.push(String::new());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_epilogue() {
        let input = SessionEpilogueInput {
            title: "Test Session".to_string(),
            session_id: Some("abc123".to_string()),
        };
        let result = session_epilogue(&input);
        assert!(result.contains("Test Session"));
        assert!(result.contains("opencode -s abc123"));
        assert!(result.contains("Session"));
        assert!(result.contains("Continue"));
    }

    #[test]
    fn test_session_epilogue_no_id() {
        let input = SessionEpilogueInput {
            title: "Test".to_string(),
            session_id: None,
        };
        let result = session_epilogue(&input);
        assert!(result.contains("opencode -s "));
    }
}
