//! Session message to LLM message conversion.
//!
//! Translates V2 Session messages into the canonical LLM `Message` format
//! that gets sent to providers. Ported from `core/src/session/runner/to-llm-message.ts`.

use opencode_schema::session::{AssistantContent, SessionMessage};
use opencode_llm::schema::{ContentPart, Message, MessageRole, Model};

pub struct MessageConverter;

impl MessageConverter {
    pub fn convert(messages: &[SessionMessage], model: &Model) -> Vec<Message> {
        messages.iter().map(|m| Self::convert_one(m, model)).collect()
    }

    fn convert_one(msg: &SessionMessage, model: &Model) -> Message {
        match msg {
            SessionMessage::User { text, files, agents: _, .. } => {
                let mut parts = vec![ContentPart::text(text.clone())];
                if let Some(files) = files {
                    for file in files {
                        parts.push(ContentPart::Media(opencode_llm::schema::MediaPart {
                            media_type: file.mime.clone(),
                            data: opencode_llm::schema::MediaData::Text(file.uri.clone()),
                            filename: file.name.clone(),
                            metadata: None,
                        }));
                    }
                }
                Message {
                    id: None,
                    role: MessageRole::User,
                    content: parts,
                    metadata: None,
                    native: None,
                }
            }
            SessionMessage::System { text, .. } => {
                Message::system(text.clone())
            }
            SessionMessage::Assistant { content, agent: _, model: msg_model, .. } => {
                let same_model = msg_model.id.0 == model.id && msg_model.provider_id.0 == model.provider;
                let parts: Vec<ContentPart> = content.iter().flat_map(|item| {
                    Self::convert_assistant_content(item, same_model)
                }).collect();
                let meaningful: Vec<ContentPart> = parts.into_iter().filter(|p| {
                    match p {
                        ContentPart::Text(t) => !t.text.is_empty(),
                        ContentPart::Reasoning(r) => !r.text.is_empty(),
                        _ => true,
                    }
                }).collect();
                Message {
                    id: None,
                    role: MessageRole::Assistant,
                    content: meaningful,
                    metadata: None,
                    native: None,
                }
            }
            SessionMessage::Synthetic { text, .. } => {
                Message::user(text.clone())
            }
            SessionMessage::Shell { command, output, .. } => {
                Message::user(format!("$ {}\n{}", command, output))
            }
            SessionMessage::AgentSwitched { agent, .. } => {
                Message::system(format!("Agent switched to: {}", agent))
            }
            SessionMessage::ModelSwitched { model, .. } => {
                Message::system(format!("Model switched to: {}:{}", model.provider_id, model.id))
            }
            SessionMessage::Compaction { summary, recent, .. } => {
                Message::system(format!("Compaction summary:\n{}\n\nRecent:\n{}", summary, recent))
            }
        }
    }

    fn convert_assistant_content(item: &AssistantContent, same_model: bool) -> Vec<ContentPart> {
        match item {
            AssistantContent::Text { text, .. } => {
                vec![ContentPart::text(text.clone())]
            }
            AssistantContent::Reasoning { text, .. } => {
                if same_model {
                    vec![ContentPart::Reasoning(opencode_llm::schema::ReasoningPart {
                        text: text.clone(),
                        encrypted: None,
                        metadata: None,
                        provider_metadata: None,
                    })]
                } else if !text.is_empty() {
                    vec![ContentPart::text(text.clone())]
                } else {
                    vec![]
                }
            }
            AssistantContent::Tool { id, name, state, .. } => {
                let input = match state {
                    opencode_schema::session::ToolState::Pending { input } => {
                        serde_json::from_str(input).unwrap_or(serde_json::Value::String(input.clone()))
                    }
                    opencode_schema::session::ToolState::Running { input, .. } => {
                        serde_json::Value::Object(serde_json::Map::from_iter(input.iter().map(|(k,v)| (k.clone(), v.clone()))))
                    }
                    opencode_schema::session::ToolState::Completed { input, .. } => {
                        serde_json::Value::Object(serde_json::Map::from_iter(input.iter().map(|(k,v)| (k.clone(), v.clone()))))
                    }
                    opencode_schema::session::ToolState::Error { input, .. } => {
                        serde_json::Value::Object(serde_json::Map::from_iter(input.iter().map(|(k,v)| (k.clone(), v.clone()))))
                    }
                };

                let tool_call = ContentPart::ToolCall(opencode_llm::schema::ToolCallPart {
                    id: id.clone(),
                    name: name.clone(),
                    input,
                    provider_executed: None,
                    metadata: None,
                    provider_metadata: None,
                });

                let mut result = vec![tool_call];

                if let opencode_schema::session::ToolState::Completed { structured, .. } = state {
                    let tool_result = ContentPart::ToolResult(opencode_llm::schema::ToolResultPart {
                        id: id.clone(),
                        name: name.clone(),
                        result: opencode_llm::schema::ToolResultValue::Json { value: serde_json::to_value(structured).unwrap_or(serde_json::Value::Null) },
                        provider_executed: None,
                        cache: None,
                        metadata: None,
                        provider_metadata: None,
                    });
                    result.push(tool_result);
                } else if let opencode_schema::session::ToolState::Error { error, .. } = state {
                    let tool_result = ContentPart::ToolResult(opencode_llm::schema::ToolResultPart {
                        id: id.clone(),
                        name: name.clone(),
                        result: opencode_llm::schema::ToolResultValue::Text { value: serde_json::to_value(&error.message).unwrap_or(serde_json::Value::Null) },
                        provider_executed: None,
                        cache: None,
                        metadata: None,
                        provider_metadata: None,
                    });
                    result.push(tool_result);
                }

                result
            }
        }
    }
}
