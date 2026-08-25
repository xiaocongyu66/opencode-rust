//! Question data models.

use serde::{Deserialize, Serialize};

use crate::ids::{QuestionID, SessionID};

/// A selectable option in a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
}

/// Question info — full question with custom-answer toggle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInfo {
    pub question: String,
    pub header: String,
    pub options: Vec<QuestionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<bool>,
}

/// A question prompt (without custom flag).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionPrompt {
    pub question: String,
    pub header: String,
    pub options: Vec<QuestionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple: Option<bool>,
}

/// Tool context for a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionTool {
    pub message_id: String,
    pub call_id: String,
}

/// A question request sent to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionRequest {
    pub id: QuestionID,
    pub session_id: SessionID,
    pub questions: Vec<QuestionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<QuestionTool>,
}

/// An answer (array of selected labels).
pub type QuestionAnswer = Vec<String>;

/// A reply to a question request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionReply {
    pub answers: Vec<QuestionAnswer>,
}
