//! Question tool — ask the user questions during execution.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct QuestionTool;

impl QuestionTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct QuestionOption {
    label: String,
    description: String,
}

#[derive(Deserialize)]
struct QuestionItem {
    question: String,
    header: String,
    options: Vec<QuestionOption>,
    #[serde(default)]
    multiple: Option<bool>,
}

#[derive(Deserialize)]
struct QuestionInput {
    questions: Vec<QuestionItem>,
}

#[async_trait]
impl Tool for QuestionTool {
    fn name(&self) -> &str { "question" }

    fn description(&self) -> &str {
        "Use this tool when you need to ask the user questions during execution. Allows gathering preferences, clarifying instructions, or getting decisions."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "question": { "type": "string" },
                            "header": { "type": "string" },
                            "options": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": { "type": "string" },
                                        "description": { "type": "string" }
                                    },
                                    "required": ["label", "description"]
                                }
                            },
                            "multiple": { "type": "boolean" }
                        },
                        "required": ["question", "header", "options"]
                    }
                }
            },
            "required": ["questions"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: QuestionInput = serde_json::from_value(params)?;
        let summary = input.questions.iter()
            .map(|q| format!("{}: {}", q.header, q.question))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(ToolResult::text(format!("Questions queued:\n{}", summary)))
    }
}
