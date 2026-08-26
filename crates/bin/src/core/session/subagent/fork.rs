//! Fork mode — byte-level context inheritance (claude-code-book Ch09).
//!
//! Fork clones the parent's message prefix so the sub-agent inherits context
//! without recomputing tokens. The fork prefix is cache-safe: the parent's
//! system prompt + conversation history up to the fork point is frozen,
//! so the sub-agent's API requests hit the prompt cache.
//!
//! Recursion guard: fork_depth counter prevents infinite sub-agent spawning.
//! Default max depth is 3 (per claude-code-book Ch09).

use crate::llm::schema::Message;

/// A fork prefix — the frozen message list the sub-agent inherits.
/// Built by cloning parent messages up to the fork point, then appending
/// a "fork marker" so the sub-agent knows it's a forked context.
#[derive(Debug, Clone)]
pub struct ForkPrefix {
    /// The cloned messages (parent history up to fork point).
    pub messages: Vec<Message>,
    /// How deep the fork chain is (parent=0, child=1, grandchild=2...).
    pub fork_depth: u32,
}

/// Build a fork prefix from parent messages. The depth is inherited from
/// the parent's fork_depth + 1. Returns None if the recursion limit is hit.
pub fn build_fork_prefix(
    parent_messages: &[Message],
    parent_depth: u32,
    max_depth: u32,
) -> Option<ForkPrefix> {
    let new_depth = parent_depth + 1;
    if new_depth > max_depth {
        tracing::warn!(
            parent_depth,
            max_depth,
            "fork recursion limit hit — refusing to spawn sub-agent"
        );
        return None;
    }
    Some(ForkPrefix {
        // Byte-level clone: the sub-agent's API call will reuse the parent's
        // prompt cache because the prefix bytes match exactly.
        messages: parent_messages.to_vec(),
        fork_depth: new_depth,
    })
}

/// Append a fork marker to the prefix so the sub-agent knows it's forked.
/// This is a synthetic system message that doesn't affect the model but
/// helps debugging and future fork-aware features.
pub fn append_fork_marker(prefix: &mut ForkPrefix, parent_agent: &str) {
    let marker = Message::system(format!(
        "[fork from {} at depth {} — inherited context, do not re-derive]",
        parent_agent, prefix.fork_depth
    ));
    prefix.messages.push(marker);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_increments_depth() {
        let parent = vec![Message::user("hello")];
        let prefix = build_fork_prefix(&parent, 0, 3).unwrap();
        assert_eq!(prefix.fork_depth, 1);
        assert_eq!(prefix.messages.len(), 1);
    }

    #[test]
    fn test_fork_respects_max_depth() {
        let parent = vec![Message::user("hello")];
        // depth 3 + 1 = 4 > 3 → reject
        assert!(build_fork_prefix(&parent, 3, 3).is_none());
        // depth 2 + 1 = 3, not > 3 → allow
        assert!(build_fork_prefix(&parent, 2, 3).is_some());
    }

    #[test]
    fn test_fork_marker_appended() {
        let parent = vec![Message::user("hello")];
        let mut prefix = build_fork_prefix(&parent, 0, 3).unwrap();
        let len_before = prefix.messages.len();
        append_fork_marker(&mut prefix, "main");
        assert_eq!(prefix.messages.len(), len_before + 1);
    }
}
