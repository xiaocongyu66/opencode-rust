//! System prompt generation.
//!
//! Ported from `session/system.ts`.
//! Generates provider-specific system prompts and assembles environment context.

/// Select the system prompt template based on the model.
pub fn select_prompt(model_id: &str, provider_id: &str) -> &'static str {
    let model_lower = model_id.to_lowercase();
    let provider_lower = provider_id.to_lowercase();

    if model_lower.contains("muse") {
        return PROMPT_META;
    }
    if model_lower.contains("gpt-4") || model_lower.contains("o1") || model_lower.contains("o3") {
        return PROMPT_BEAST;
    }
    if model_lower.contains("gpt") {
        if model_lower.contains("codex") {
            return PROMPT_CODEX;
        }
        return PROMPT_GPT;
    }
    if model_lower.contains("gemini-") {
        return PROMPT_GEMINI;
    }
    if model_lower.contains("claude") {
        return PROMPT_ANTHROPIC;
    }
    if model_lower.contains("trinity") {
        return PROMPT_TRINITY;
    }
    if model_lower.contains("kimi")
        || provider_lower.contains("kimi")
        || provider_lower.contains("moonshot")
    {
        return PROMPT_KIMI;
    }
    PROMPT_DEFAULT
}

/// Build the environment info block.
pub fn environment_block(
    model_id: &str,
    provider_id: &str,
    directory: &str,
    worktree: &str,
    is_git: bool,
    platform: &str,
    date: &str,
) -> String {
    format!(
        "You are powered by the model named {model_id}. The exact model ID is {provider_id}/{model_id}\n\
         Here is some useful information about the environment you are running in:\n\
         <env>\n\
         \x20 Working directory: {directory}\n\
         \x20 Workspace root folder: {worktree}\n\
         \x20 Is directory a git repo: {is_git}\n\
         \x20 Platform: {platform}\n\
         \x20 Today's date: {date}\n\
         </env>",
        model_id = model_id,
        provider_id = provider_id,
        directory = directory,
        worktree = worktree,
        is_git = if is_git { "yes" } else { "no" },
        platform = platform,
        date = date,
    )
}

const PROMPT_DEFAULT: &str = include_str!("prompt/default.txt");
const PROMPT_ANTHROPIC: &str = include_str!("prompt/anthropic.txt");
const PROMPT_BEAST: &str = include_str!("prompt/beast.txt");
const PROMPT_GEMINI: &str = include_str!("prompt/gemini.txt");
const PROMPT_GPT: &str = include_str!("prompt/gpt.txt");
const PROMPT_KIMI: &str = include_str!("prompt/kimi.txt");
const PROMPT_META: &str = include_str!("prompt/meta.txt");
const PROMPT_CODEX: &str = include_str!("prompt/codex.txt");
const PROMPT_TRINITY: &str = include_str!("prompt/trinity.txt");
