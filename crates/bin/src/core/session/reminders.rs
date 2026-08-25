//! Session reminders.
//!
//! Ported from `session/reminders.ts`.
//! Injects plan-mode and build-switch reminders into user messages.

use crate::schema::session::SessionMessage;

const PROMPT_PLAN: &str = include_str!("prompt/plan.txt");
const BUILD_SWITCH: &str = include_str!("prompt/build-switch.txt");
const PLAN_MODE: &str = include_str!("prompt/plan-mode.txt");

/// Apply reminders to messages based on agent mode.
pub fn apply(
    messages: &mut Vec<SessionMessage>,
    agent_name: &str,
) {
    let was_plan = messages.iter().any(|m| {
        matches!(m, SessionMessage::Assistant { agent, .. } if agent == "plan")
    });

    let user_msg = match messages
        .iter_mut()
        .rev()
        .find(|m| matches!(m, SessionMessage::User { .. }))
    {
        Some(m) => m,
        None => return,
    };

    if agent_name == "plan" {
        inject_text(user_msg, PROMPT_PLAN.to_string());
        return;
    }

    if was_plan && agent_name == "build" {
        inject_text(user_msg, BUILD_SWITCH.to_string());
    }
}

/// Apply plan mode reminder.
pub fn apply_plan_mode(
    messages: &mut Vec<SessionMessage>,
    agent_name: &str,
    plan_path: &str,
    plan_exists: bool,
) {
    let assistant_agent = messages
        .iter()
        .rev()
        .find(|m| matches!(m, SessionMessage::Assistant { .. }))
        .and_then(|m| {
            if let SessionMessage::Assistant { agent, .. } = m {
                Some(agent.clone())
            } else {
                None
            }
        });

    let user_msg = match messages
        .iter_mut()
        .rev()
        .find(|m| matches!(m, SessionMessage::User { .. }))
    {
        Some(m) => m,
        None => return,
    };

    if agent_name != "plan" && assistant_agent.as_deref() == Some("plan") {
        let text = if plan_exists {
            format!(
                "{}\n\nA plan file exists at {}. You should execute on the plan defined within it",
                BUILD_SWITCH, plan_path
            )
        } else {
            BUILD_SWITCH.to_string()
        };
        inject_text(user_msg, text);
        return;
    }

    if agent_name != "plan" || assistant_agent.as_deref() == Some("plan") {
        return;
    }

    let plan_info = if plan_exists {
        format!(
            "A plan file already exists at {}. You can read it and make incremental edits using the edit tool.",
            plan_path
        )
    } else {
        format!(
            "No plan file exists yet. You should create your plan at {} using the write tool.",
            plan_path
        )
    };

    let text = PLAN_MODE.replace("${planInfo}", &plan_info);
    inject_text(user_msg, text);
}

fn inject_text(msg: &mut SessionMessage, text: String) {
    if let SessionMessage::User { text: existing, .. } = msg {
        if !existing.is_empty() {
            existing.push('\n');
        }
        existing.push_str(&text);
    }
}
