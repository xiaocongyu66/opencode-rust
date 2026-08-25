//! Instruction context — builds system prompts and context for LLM requests.

use std::collections::HashMap;

pub struct InstructionContext {
    pub system_prompt: String,
    pub agent_instructions: HashMap<String, String>,
    pub tool_descriptions: HashMap<String, String>,
    pub references: Vec<String>,
}

impl InstructionContext {
    pub fn new() -> Self {
        Self {
            system_prompt: String::new(),
            agent_instructions: HashMap::new(),
            tool_descriptions: HashMap::new(),
            references: vec![],
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    pub fn add_reference(&mut self, reference: impl Into<String>) {
        self.references.push(reference.into());
    }

    pub fn build(&self) -> String {
        let mut parts = vec![self.system_prompt.clone()];
        if !self.references.is_empty() {
            parts.push(format!("\n## References\n{}", self.references.join("\n")));
        }
        parts.join("\n")
    }
}

impl Default for InstructionContext {
    fn default() -> Self {
        Self::new()
    }
}
