//! AskUserQuestion tool — ask the user a multiple-choice question.
//!
//! Aligned with claude-code-best AskUserQuestionTool:
//! - `questions` (required): array of { question, header, options, multiSelect }
//!   - options: array of { label, description, preview? }

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct AskUserQuestionTool;

impl AskUserQuestionTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct AskUserQuestionInput {
    questions: Vec<Question>,
}

#[derive(Deserialize)]
struct Question {
    question: String,
    header: String,
    options: Vec<QuestionOption>,
    #[serde(default, rename = "multiSelect")]
    multi_select: Option<bool>,
}

#[derive(Deserialize)]
struct QuestionOption {
    label: String,
    description: String,
    #[serde(default)]
    preview: Option<String>,
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    fn name(&self) -> &str { "AskUserQuestion"
    }

    fn description(&self) -> &str {
        "Asks the user a multiple-choice question. Use when you need \
         clarification or a decision between options. Each question has a \
         header (short label), the question text, and 2+ options."
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
                            "question": { "type": "string", "description": "The complete question to ask the user." },
                            "header": { "type": "string", "description": "Very short label displayed as a chip (max 25 chars)." },
                            "options": {
                                "type": "array",
                                "minItems": 2,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": { "type": "string", "description": "Display text for this option (1-5 words)." },
                                        "description": { "type": "string", "description": "Explanation of what this option means." },
                                        "preview": { "type": "string", "description": "Optional preview content rendered when focused." }
                                    },
                                    "required": ["label", "description"]
                                }
                            },
                            "multiSelect": { "type": "boolean", "description": "If true, user can select multiple options." }
                        },
                        "required": ["question", "header", "options"]
                    }
                }
            },
            "required": ["questions"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: AskUserQuestionInput = serde_json::from_value(params)?;
        // In TUI mode this would prompt the user interactively. For now,
        // we return the question text so the runner/app can pick it up
        // and route it to the TUI's question prompt.
        let mut out = String::new();
        for (i, q) in input.questions.iter().enumerate() {
            out.push_str(&format!("Q{} [{}]: {}\n", i + 1, q.header, q.question));
            for (j, opt) in q.options.iter().enumerate() {
                out.push_str(&format!(
                    "  {}. {} — {}\n",
                    j + 1,
                    opt.label,
                    opt.description
                ));
            }
            out.push('\n');
        }
        // Note: in a real implementation, ctx would carry a channel back to
        // the TUI's question prompt. For now we just return the question.
        let _ = ctx;
        Ok(ToolResult::text(format!(
            "Question for user:\n{}\n(Interactive prompt not yet wired — returning question text)",
            out
        )))
    }
}
