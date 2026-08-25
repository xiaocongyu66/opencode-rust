use serde_json::Value;
use std::collections::HashMap;

use super::locale;
use super::model;

#[derive(Clone, Debug)]
pub struct TranscriptOptions {
    pub thinking: bool,
    pub tool_details: bool,
    pub assistant_metadata: bool,
    pub providers: Vec<model::Provider>,
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub time_created: i64,
    pub time_updated: i64,
}

#[derive(Clone, Debug)]
pub struct MessageWithParts {
    pub info: MessageInfo,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug)]
pub struct MessageInfo {
    pub id: String,
    pub role: String,
    pub agent: String,
    pub provider_id: String,
    pub model_id: String,
    pub time_created: i64,
    pub time_completed: Option<i64>,
}

#[derive(Clone, Debug)]
pub enum Part {
    Text {
        text: String,
        synthetic: bool,
    },
    Reasoning {
        text: String,
    },
    Tool {
        tool: String,
        state: ToolState,
    },
}

#[derive(Clone, Debug)]
pub struct ToolState {
    pub status: String,
    pub input: Option<Value>,
    pub output: Option<String>,
    pub error: Option<String>,
}

pub fn format_transcript(
    session: &SessionInfo,
    messages: &[MessageWithParts],
    options: &TranscriptOptions,
) -> String {
    let providers = model::index(Some(&options.providers));
    let mut transcript = format!("# {}\n\n", session.title);
    transcript.push_str(&format!("**Session ID:** {}\n", session.id));
    transcript.push_str(&format!("**Created:** {}\n", format_timestamp(session.time_created)));
    transcript.push_str(&format!("**Updated:** {}\n\n", format_timestamp(session.time_updated)));
    transcript.push_str("---\n\n");

    let mut sorted: Vec<&MessageWithParts> = messages.iter().collect();
    sorted.sort_by(|a, b| {
        a.info
            .time_created
            .cmp(&b.info.time_created)
            .then(a.info.id.cmp(&b.info.id))
    });

    for msg in &sorted {
        transcript.push_str(&format_message(&msg.info, &msg.parts, options, Some(&providers)));
        transcript.push_str("---\n\n");
    }
    transcript
}

pub fn format_message(
    msg: &MessageInfo,
    parts: &[Part],
    options: &TranscriptOptions,
    providers: Option<&HashMap<String, model::Provider>>,
) -> String {
    let mut result = String::new();
    if msg.role == "user" {
        result.push_str("## User\n\n");
    } else {
        result.push_str(&format_assistant_header(
            msg,
            options.assistant_metadata,
            providers,
        ));
    }
    for part in parts {
        result.push_str(&format_part(part, options));
    }
    result
}

pub fn format_assistant_header(
    msg: &MessageInfo,
    include_metadata: bool,
    providers: Option<&HashMap<String, model::Provider>>,
) -> String {
    if !include_metadata {
        return "## Assistant\n\n".to_string();
    }
    let duration = match (msg.time_completed, msg.time_created) {
        (Some(completed), created) if completed > 0 && created > 0 => {
            format!("{:.1}s", (completed - created) as f64 / 1000.0)
        }
        _ => String::new(),
    };
    let model_name = model::name(None, providers, &msg.provider_id, &msg.model_id);
    let agent_title = locale::titlecase(&msg.agent);
    if duration.is_empty() {
        format!("## Assistant ({} · {})\n\n", agent_title, model_name)
    } else {
        format!(
            "## Assistant ({} · {} · {})\n\n",
            agent_title, model_name, duration
        )
    }
}

pub fn format_part(part: &Part, options: &TranscriptOptions) -> String {
    match part {
        Part::Text { text, synthetic } => {
            if !synthetic {
                format!("{}\n\n", text)
            } else {
                String::new()
            }
        }
        Part::Reasoning { text } => {
            if options.thinking {
                format!("_Thinking:_\n\n{}\n\n", text)
            } else {
                String::new()
            }
        }
        Part::Tool { tool, state } => {
            let mut result = format!("**Tool: {}**\n", tool);
            if options.tool_details {
                if let Some(input) = &state.input {
                    let json_str = serde_json::to_string_pretty(input).unwrap_or_default();
                    result.push_str(&format!("\n**Input:**\n```json\n{}\n```\n", json_str));
                }
                if state.status == "completed" {
                    if let Some(output) = &state.output {
                        result.push_str(&format!("\n**Output:**\n```\n{}\n```\n", output));
                    }
                }
                if state.status == "error" {
                    if let Some(error) = &state.error {
                        result.push_str(&format!("\n**Error:**\n```\n{}\n```\n", error));
                    }
                }
            }
            result.push('\n');
            result
        }
    }
}

fn format_timestamp(millis: i64) -> String {
    use chrono::{DateTime, Local, Utc};
    DateTime::<Utc>::from_timestamp_millis(millis)
        .unwrap_or_default()
        .with_timezone(&Local)
        .to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_options() -> TranscriptOptions {
        TranscriptOptions {
            thinking: true,
            tool_details: true,
            assistant_metadata: true,
            providers: vec![],
        }
    }

    #[test]
    fn test_format_text_part() {
        let part = Part::Text {
            text: "Hello".to_string(),
            synthetic: false,
        };
        let result = format_part(&part, &make_options());
        assert_eq!(result, "Hello\n\n");
    }

    #[test]
    fn test_format_synthetic_text() {
        let part = Part::Text {
            text: "System".to_string(),
            synthetic: true,
        };
        let result = format_part(&part, &make_options());
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_reasoning() {
        let part = Part::Reasoning {
            text: "Hmm".to_string(),
        };
        let result = format_part(&part, &make_options());
        assert!(result.contains("_Thinking:_"));
        assert!(result.contains("Hmm"));
    }

    #[test]
    fn test_format_reasoning_disabled() {
        let mut options = make_options();
        options.thinking = false;
        let part = Part::Reasoning {
            text: "Hmm".to_string(),
        };
        let result = format_part(&part, &options);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_tool_completed() {
        let part = Part::Tool {
            tool: "bash".to_string(),
            state: ToolState {
                status: "completed".to_string(),
                input: Some(json!({"command": "ls"})),
                output: Some("file1\nfile2".to_string()),
                error: None,
            },
        };
        let result = format_part(&part, &make_options());
        assert!(result.contains("**Tool: bash**"));
        assert!(result.contains("**Input:**"));
        assert!(result.contains("\"command\""));
        assert!(result.contains("**Output:**"));
        assert!(result.contains("file1"));
    }

    #[test]
    fn test_format_tool_error() {
        let part = Part::Tool {
            tool: "read".to_string(),
            state: ToolState {
                status: "error".to_string(),
                input: None,
                output: None,
                error: Some("File not found".to_string()),
            },
        };
        let result = format_part(&part, &make_options());
        assert!(result.contains("**Error:**"));
        assert!(result.contains("File not found"));
    }

    #[test]
    fn test_format_assistant_header_no_metadata() {
        let msg = MessageInfo {
            id: "1".to_string(),
            role: "assistant".to_string(),
            agent: "coder".to_string(),
            provider_id: "test".to_string(),
            model_id: "model-1".to_string(),
            time_created: 0,
            time_completed: None,
        };
        let header = format_assistant_header(&msg, false, None);
        assert_eq!(header, "## Assistant\n\n");
    }

    #[test]
    fn test_format_assistant_header_with_metadata() {
        let msg = MessageInfo {
            id: "1".to_string(),
            role: "assistant".to_string(),
            agent: "coder".to_string(),
            provider_id: "test".to_string(),
            model_id: "model-1".to_string(),
            time_created: 1000,
            time_completed: Some(2500),
        };
        let header = format_assistant_header(&msg, true, None);
        assert!(header.contains("Coder"));
        assert!(header.contains("model-1"));
        assert!(header.contains("1.5s"));
    }

    #[test]
    fn test_format_user_message() {
        let msg = MessageInfo {
            id: "1".to_string(),
            role: "user".to_string(),
            agent: "".to_string(),
            provider_id: "".to_string(),
            model_id: "".to_string(),
            time_created: 0,
            time_completed: None,
        };
        let parts = vec![Part::Text {
            text: "Hello".to_string(),
            synthetic: false,
        }];
        let result = format_message(&msg, &parts, &make_options(), None);
        assert!(result.contains("## User"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_format_transcript() {
        let session = SessionInfo {
            id: "session-1".to_string(),
            title: "Test Session".to_string(),
            time_created: 1700000000000,
            time_updated: 1700000001000,
        };
        let messages = vec![MessageWithParts {
            info: MessageInfo {
                id: "msg-1".to_string(),
                role: "user".to_string(),
                agent: "".to_string(),
                provider_id: "".to_string(),
                model_id: "".to_string(),
                time_created: 1700000000000,
                time_completed: None,
            },
            parts: vec![Part::Text {
                text: "Hi".to_string(),
                synthetic: false,
            }],
        }];
        let result = format_transcript(&session, &messages, &make_options());
        assert!(result.contains("# Test Session"));
        assert!(result.contains("**Session ID:** session-1"));
        assert!(result.contains("---"));
        assert!(result.contains("## User"));
        assert!(result.contains("Hi"));
    }
}
