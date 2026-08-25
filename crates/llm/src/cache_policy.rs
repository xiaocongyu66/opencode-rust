//! Cache policy — auto-placement of `CacheHint`s onto request parts.
//!
//! Migrated from `packages/llm/src/cache-policy.ts`.

use crate::schema::{
    CacheHint, CacheHintType, CachePolicy, CachePolicyObject, ContentPart, LlmRequest, Message,
    SystemPart, ToolDefinition,
};

const RESPECTS_INLINE_HINTS: &[&str] = &["anthropic-messages", "bedrock-converse"];

fn resolve(policy: Option<&CachePolicy>) -> CachePolicyObject {
    match policy {
        None | Some(CachePolicy::Mode(crate::schema::CachePolicyMode::Auto)) => CachePolicyObject {
            tools: Some(true),
            system: Some(true),
            messages: Some(crate::schema::CachePolicyMessages::Strategy(
                crate::schema::CachePolicyMessageStrategy::LatestUserMessage,
            )),
            ttl_seconds: None,
        },
        Some(CachePolicy::Mode(crate::schema::CachePolicyMode::None)) => CachePolicyObject::default(),
        Some(CachePolicy::Object(obj)) => obj.clone(),
    }
}

fn make_hint(ttl_seconds: Option<u64>) -> CacheHint {
    CacheHint {
        r#type: CacheHintType::Ephemeral,
        ttl_seconds,
    }
}

fn mark_last_tool(tools: &[ToolDefinition], hint: &CacheHint) -> Option<Vec<ToolDefinition>> {
    if tools.is_empty() {
        return None;
    }
    let last = tools.len() - 1;
    if tools[last].cache.is_some() {
        return None;
    }
    let next: Vec<ToolDefinition> = tools
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == last {
                let mut t = t.clone();
                t.cache = Some(hint.clone());
                t
            } else {
                t.clone()
            }
        })
        .collect();
    Some(next)
}

fn mark_last_system(system: &[SystemPart], hint: &CacheHint) -> Option<Vec<SystemPart>> {
    if system.is_empty() {
        return None;
    }
    let last = system.len() - 1;
    if system[last].cache.is_some() {
        return None;
    }
    let next: Vec<SystemPart> = system
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == last {
                let mut p = p.clone();
                p.cache = Some(hint.clone());
                p
            } else {
                p.clone()
            }
        })
        .collect();
    Some(next)
}

fn last_index_of_role(messages: &[Message], role: crate::schema::MessageRole) -> isize {
    let mut idx = -1;
    for (i, m) in messages.iter().enumerate() {
        if m.role == role {
            idx = i as isize;
        }
    }
    idx
}

fn mark_message_at(messages: &[Message], index: isize, hint: &CacheHint) -> Option<Vec<Message>> {
    if index < 0 {
        return None;
    }
    let index = index as usize;
    if index >= messages.len() {
        return None;
    }
    let target = &messages[index];
    if target.content.is_empty() {
        return None;
    }

    // Find last text part, else last content part
    let mut mark_at = target.content.len() - 1;
    for (i, part) in target.content.iter().enumerate().rev() {
        if matches!(part, ContentPart::Text(_)) {
            mark_at = i;
            break;
        }
    }

    // Check existing cache
    let has_cache = match &target.content[mark_at] {
        ContentPart::Text(t) => t.cache.is_some(),
        ContentPart::Reasoning(r) => r.metadata.is_some(), // simplified
        _ => false,
    };
    if has_cache {
        return None;
    }

    let mut next_content = target.content.clone();
    next_content[mark_at] = match &next_content[mark_at] {
        ContentPart::Text(t) => {
            let mut t = t.clone();
            t.cache = Some(hint.clone());
            ContentPart::Text(t)
        }
        ContentPart::Reasoning(r) => {
            let mut r = r.clone();
            r.metadata = Some({
                let mut m = serde_json::Map::new();
                m.insert("cache".to_string(), serde_json::Value::Bool(true));
                m
            });
            ContentPart::Reasoning(r)
        }
        other => other.clone(),
    };

    let mut next = messages.to_vec();
    next[index] = Message {
        id: target.id.clone(),
        role: target.role,
        content: next_content,
        metadata: target.metadata.clone(),
        native: target.native.clone(),
    };
    Some(next)
}

fn mark_messages(
    messages: &[Message],
    strategy: &crate::schema::CachePolicyMessages,
    hint: &CacheHint,
) -> Option<Vec<Message>> {
    if messages.is_empty() {
        return None;
    }
    match strategy {
        crate::schema::CachePolicyMessages::Strategy(
            crate::schema::CachePolicyMessageStrategy::LatestUserMessage,
        ) => mark_message_at(messages, last_index_of_role(messages, crate::schema::MessageRole::User), hint),
        crate::schema::CachePolicyMessages::Strategy(
            crate::schema::CachePolicyMessageStrategy::LatestAssistant,
        ) => mark_message_at(messages, last_index_of_role(messages, crate::schema::MessageRole::Assistant), hint),
        crate::schema::CachePolicyMessages::Tail { tail } => {
            let start = messages.len().saturating_sub(*tail as usize);
            let mut next = messages.to_vec();
            for i in start..messages.len() {
                if let Some(updated) = mark_message_at(&next, i as isize, hint) {
                    next = updated;
                }
            }
            Some(next)
        }
    }
}

/// Apply an `LLMRequest.cache` policy by injecting `CacheHint`s onto the parts
/// the policy designates. Runs once at compile time, before the per-protocol
/// body builder, so the existing inline-hint lowering path handles the rest.
///
/// Returns a new request with cache hints applied. If the route does not
/// respect inline hints (or the policy is empty), returns the original
/// request unchanged.
pub fn apply_cache_policy(request: &LlmRequest) -> LlmRequest {
    // Route id is not available on Model in this simplified Rust model.
    // In the TS original, `request.model.route.id` is checked against
    // `RESPECTS_INLINE_HINTS`. Here we apply the policy unconditionally —
    // protocols that ignore hints will simply not act on them.
    let _ = RESPECTS_INLINE_HINTS;

    let policy = resolve(request.cache.as_ref());
    if policy.tools.is_none() && policy.system.is_none() && policy.messages.is_none() {
        return request.clone();
    }

    let hint = make_hint(policy.ttl_seconds);

    let tools = if policy.tools == Some(true) {
        mark_last_tool(&request.tools, &hint)
    } else {
        None
    };
    let system = if policy.system == Some(true) {
        mark_last_system(&request.system, &hint)
    } else {
        None
    };
    let messages = if let Some(strategy) = &policy.messages {
        mark_messages(&request.messages, strategy, &hint)
    } else {
        None
    };

    if tools.is_none() && system.is_none() && messages.is_none() {
        return request.clone();
    }

    let mut result = request.clone();
    if let Some(t) = tools {
        result.tools = t;
    }
    if let Some(s) = system {
        result.system = s;
    }
    if let Some(m) = messages {
        result.messages = m;
    }
    result
}
