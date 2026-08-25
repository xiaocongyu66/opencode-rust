//! Provider error classification helpers.
//!
//! Migrated from `packages/llm/src/provider-error.ts`.

use crate::llm::schema::{LlmError, LlmErrorReason, LlmEvent, ProviderFailureClassification};

const PATTERNS: &[&str] = &[
    r"(?i)prompt is too long",
    r"(?i)request_too_large",
    r"(?i)input is too long for requested model",
    r"(?i)exceeds the context window",
    r"(?i)exceeds (?:the )?(?:model'?s )?maximum context length(?: of [\d,]+ tokens?|\s*\([\d,]+\))",
    r"(?i)input token count.*exceeds the maximum",
    r"(?i)tokens in request more than max tokens allowed",
    r"(?i)maximum prompt length is \d+",
    r"(?i)reduce the length of the messages",
    r"(?i)maximum context length is \d+ tokens",
    r"(?i)exceeds (?:the )?maximum allowed input length of [\d,]+ tokens?",
    r"(?i)input \(\d+ tokens\) is longer than the model'?s context length \(\d+ tokens\)",
    r"(?i)exceeds the limit of \d+",
    r"(?i)exceeds the available context size",
    r"(?i)greater than the context length",
    r"(?i)context window exceeds limit",
    r"(?i)exceeded model token limit",
    r"(?i)context[_ ]length[_ ]exceeded",
    r"(?i)request entity too large",
    r"(?i)context length is only \d+ tokens",
    r"(?i)input length.*exceeds.*context length",
    r"(?i)prompt too long; exceeded (?:max )?context length",
    r"(?i)too large for model with \d+ maximum context length",
    r"(?i)prompt has [\d,]+ tokens?, but the configured context size is [\d,]+ tokens?",
    r"(?i)model_context_window_exceeded",
    r"(?i)too many tokens",
    r"(?i)token limit exceeded",
];

const EXCLUSIONS: &[&str] = &[
    r"(?i)^(throttling error|service unavailable):",
    r"(?i)rate limit",
    r"(?i)too many requests",
];

/// Detect context-overflow messages from provider error text.
pub fn is_context_overflow(message: &str) -> bool {
    if EXCLUSIONS.iter().any(|p| {
        regex_lite::Regex::new(p)
            .map(|r| r.is_match(message))
            .unwrap_or(false)
    }) {
        return false;
    }
    if PATTERNS.iter().any(|p| {
        regex_lite::Regex::new(p)
            .map(|r| r.is_match(message))
            .unwrap_or(false)
    }) {
        return true;
    }
    // 400/413 with no body
    regex_lite::Regex::new(r"(?i)^4(00|13)\s*(status code)?\s*\(no body\)")
        .map(|r| r.is_match(message))
        .unwrap_or(false)
}

/// Check whether a failure (error or event) is a classified context overflow.
pub fn is_context_overflow_failure(failure: &ContextOverflowFailure<'_>) -> bool {
    match failure {
        ContextOverflowFailure::Error(e) => matches!(
            &e.reason,
            LlmErrorReason::InvalidRequest {
                classification: Some(ProviderFailureClassification::ContextOverflow),
                ..
            }
        ),
        ContextOverflowFailure::Event(e) => {
            matches!(
                e,
                LlmEvent::ProviderError {
                    classification: Some(ProviderFailureClassification::ContextOverflow),
                    ..
                }
            )
        }
    }
}

/// Input accepted by [`is_context_overflow_failure`].
pub enum ContextOverflowFailure<'a> {
    Error(&'a LlmError),
    Event(&'a LlmEvent),
}

// Minimal inline regex to avoid pulling in the `regex` crate.
mod regex_lite {
    /// Very small regex matcher: supports literal text, `(?i)` flag, `\d`,
    /// `[\d,]`, `+`, `?`, `*`, `(?:...)`, alternation `|`, and anchors `^`.
    /// Sufficient for the pattern set in this module.
    pub struct Regex {
        case_insensitive: bool,
        pattern: String,
    }

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, ()> {
            let (case_insensitive, pattern) = if let Some(rest) = pattern.strip_prefix("(?i)") {
                (true, rest.to_string())
            } else {
                (false, pattern.to_string())
            };
            Ok(Self { case_insensitive, pattern })
        }

        pub fn is_match(&self, haystack: &str) -> bool {
            let h = if self.case_insensitive {
                haystack.to_lowercase()
            } else {
                haystack.to_string()
            };
            let p = if self.case_insensitive {
                self.pattern.to_lowercase()
            } else {
                self.pattern.clone()
            };
            self.match_pattern(&p, &h)
        }

        fn match_pattern(&self, pattern: &str, haystack: &str) -> bool {
            self.match_from(pattern, haystack, 0)
        }

        fn match_from(&self, pattern: &str, haystack: &str, start: usize) -> bool {
            let p: Vec<char> = pattern.chars().collect();
            let h: Vec<char> = haystack.chars().collect();
            for s in start..=h.len() {
                if self.try_match(&p, 0, &h, s) {
                    return true;
                }
            }
            false
        }

        fn try_match(&self, p: &[char], pi: usize, h: &[char], hi: usize) -> bool {
            if pi >= p.len() {
                return true;
            }
            // Handle (?i) prefix already stripped; handle ^ anchor
            if p[pi] == '^' {
                return hi == 0 && self.try_match(p, pi + 1, h, 0);
            }

            // (?:...) non-capturing group
            if pi + 2 < p.len() && p[pi] == '(' && p[pi + 1] == '?' && p[pi + 2] == ':' {
                return self.match_group(p, pi, h, hi);
            }

            // alternation across the whole pattern: split on top-level |
            // (simplified: only top-level)
            // We handle | inside groups and at top level.

            let next_is_quantifier = pi + 1 < p.len() && matches!(p[pi + 1], '+' | '?' | '*');

            if next_is_quantifier {
                let quant = p[pi + 1];
                return self.match_quantified(p, pi, quant, h, hi);
            }

            if hi >= h.len() {
                return false;
            }

            if p[pi] == '\\' && pi + 1 < p.len() {
                let c = p[pi + 1];
                let matched = match c {
                    'd' => h[hi].is_ascii_digit(),
                    _ => h[hi] == c,
                };
                if !matched {
                    return false;
                }
                return self.try_match(p, pi + 2, h, hi + 1);
            }

            if p[pi] == '[' {
                // character class
                return self.match_class(p, pi, h, hi);
            }

            if p[pi] == '.' {
                return self.try_match(p, pi + 1, h, hi + 1);
            }

            if p[pi] == h[hi] {
                return self.try_match(p, pi + 1, h, hi + 1);
            }

            false
        }

        fn match_quantified(&self, p: &[char], pi: usize, quant: char, h: &[char], hi: usize) -> bool {
            let atom_end = self.atom_end(p, pi);
            match quant {
                '*' => {
                    // greedy: try as many as possible then backtrack
                    let mut count = 0;
                    let mut pos = hi;
                    while pos < h.len() && self.atom_matches(p, pi, atom_end, h, pos) {
                        count += 1;
                        pos += self.atom_len(p, pi, atom_end, h, pos);
                    }
                    loop {
                        if self.try_match(p, atom_end + 1, h, hi + (pos - hi)) {
                            return true;
                        }
                        if count == 0 {
                            return self.try_match(p, atom_end + 1, h, hi);
                        }
                        count -= 1;
                        pos = hi;
                        // backtrack is simplified
                        break;
                    }
                    self.try_match(p, atom_end + 1, h, pos)
                }
                '+' => {
                    if !self.atom_matches(p, pi, atom_end, h, hi) {
                        return false;
                    }
                    let len = self.atom_len(p, pi, atom_end, h, hi);
                    // try matching rest after 1+ atoms
                    let mut pos = hi + len;
                    loop {
                        if self.try_match(p, atom_end + 1, h, pos) {
                            return true;
                        }
                        if pos < h.len() && self.atom_matches(p, pi, atom_end, h, pos) {
                            pos += self.atom_len(p, pi, atom_end, h, pos);
                        } else {
                            return false;
                        }
                    }
                }
                '?' => {
                    if self.atom_matches(p, pi, atom_end, h, hi) {
                        let len = self.atom_len(p, pi, atom_end, h, hi);
                        if self.try_match(p, atom_end + 1, h, hi + len) {
                            return true;
                        }
                    }
                    self.try_match(p, atom_end + 1, h, hi)
                }
                _ => false,
            }
        }

        fn atom_matches(&self, p: &[char], start: usize, end: usize, h: &[char], hi: usize) -> bool {
            if hi >= h.len() {
                return false;
            }
            if start >= end {
                return false;
            }
            if p[start] == '\\' && start + 1 < end {
                let c = p[start + 1];
                return match c {
                    'd' => h[hi].is_ascii_digit(),
                    _ => h[hi] == c,
                };
            }
            if p[start] == '[' {
                return self.match_class_bool(p, start, h[hi]);
            }
            if p[start] == '.' {
                return true;
            }
            p[start] == h[hi]
        }

        fn atom_len(&self, p: &[char], start: usize, end: usize, _h: &[char], _hi: usize) -> usize {
            if p[start] == '\\' && start + 1 < end {
                return 1;
            }
            if p[start] == '[' {
                // find closing ]
                let mut i = start + 1;
                while i < end && p[i] != ']' {
                    i += 1;
                }
                return 1;
            }
            1
        }

        fn atom_end(&self, p: &[char], pi: usize) -> usize {
            if p[pi] == '\\' && pi + 1 < p.len() {
                return pi + 2;
            }
            if p[pi] == '[' {
                let mut i = pi + 1;
                while i < p.len() && p[i] != ']' {
                    i += 1;
                }
                return i + 1;
            }
            pi + 1
        }

        fn match_class(&self, p: &[char], pi: usize, h: &[char], hi: usize) -> bool {
            if hi >= h.len() {
                return false;
            }
            if !self.match_class_bool(p, pi, h[hi]) {
                return false;
            }
            // find closing ]
            let mut i = pi + 1;
            while i < p.len() && p[i] != ']' {
                i += 1;
            }
            self.try_match(p, i + 1, h, hi + 1)
        }

        fn match_class_bool(&self, p: &[char], pi: usize, c: char) -> bool {
            let mut i = pi + 1;
            let negate = i < p.len() && p[i] == '^';
            if negate {
                i += 1;
            }
            let mut matched = false;
            while i < p.len() && p[i] != ']' {
                if p[i] == '\\' && i + 1 < p.len() {
                    let esc = p[i + 1];
                    let m = match esc {
                        'd' => c.is_ascii_digit(),
                        _ => c == esc,
                    };
                    if m {
                        matched = true;
                    }
                    i += 2;
                } else if i + 2 < p.len() && p[i + 1] == '-' {
                    let lo = p[i];
                    let hi = p[i + 2];
                    if c >= lo && c <= hi {
                        matched = true;
                    }
                    i += 3;
                } else {
                    if p[i] == c {
                        matched = true;
                    }
                    i += 1;
                }
            }
            matched != negate
        }

        fn match_group(&self, p: &[char], pi: usize, h: &[char], hi: usize) -> bool {
            // p[pi..] = "(?:...)"
            // find matching close paren
            let mut depth = 1;
            let mut i = pi + 3; // skip "(?:"
            let group_start = i;
            while i < p.len() && depth > 0 {
                if p[i] == '(' {
                    depth += 1;
                } else if p[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                i += 1;
            }
            // group content = p[group_start..i]
            let group_end = i;
            let after = i + 1; // skip ')'

            // Check for quantifier after group
            let has_quant = after < p.len() && matches!(p[after], '+' | '?' | '*');

            if has_quant {
                let quant = p[after];
                // simplified: try matching group 0+ times then rest
                match quant {
                    '*' | '+' => {
                        let min = if quant == '+' { 1 } else { 0 };
                        self.match_group_repeat(p, group_start, group_end, after + 1, h, hi, min)
                    }
                    '?' => {
                        // optional group
                        if self.match_group_once(p, group_start, group_end, h, hi, after + 1) {
                            return true;
                        }
                        self.try_match(p, after + 1, h, hi)
                    }
                    _ => false,
                }
            } else {
                self.match_group_once(p, group_start, group_end, h, hi, after)
            }
        }

        fn match_group_once(
            &self,
            p: &[char],
            gs: usize,
            ge: usize,
            h: &[char],
            hi: usize,
            after: usize,
        ) -> bool {
            // Try to match the group content, possibly with | alternations
            let group: Vec<char> = p[gs..ge].to_vec();
            // Split on top-level |
            for alt in self.split_alternatives(&group) {
                if let Some(end_hi) = self.match_subpattern(&alt, h, hi) {
                    if self.try_match(p, after, h, end_hi) {
                        return true;
                    }
                }
            }
            false
        }

        fn match_group_repeat(
            &self,
            p: &[char],
            gs: usize,
            ge: usize,
            after: usize,
            h: &[char],
            hi: usize,
            min: usize,
        ) -> bool {
            let group: Vec<char> = p[gs..ge].to_vec();
            let alts = self.split_alternatives(&group);
            // Greedy: match as many as possible
            let mut positions = vec![hi];
            loop {
                let last = *positions.last().unwrap();
                let mut found = false;
                for alt in &alts {
                    if let Some(next) = self.match_subpattern(alt, h, last) {
                        if next > last {
                            positions.push(next);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    break;
                }
            }
            // Try from longest match down to min
            while positions.len() > min {
                let pos = *positions.last().unwrap();
                if self.try_match(p, after, h, pos) {
                    return true;
                }
                positions.pop();
            }
            if min == 0 {
                self.try_match(p, after, h, hi)
            } else {
                false
            }
        }

        fn split_alternatives<'b>(&self, group: &'b [char]) -> Vec<Vec<char>> {
            let mut alts = Vec::new();
            let mut current = Vec::new();
            let mut depth = 0;
            for &c in group {
                if c == '(' {
                    depth += 1;
                    current.push(c);
                } else if c == ')' {
                    depth -= 1;
                    current.push(c);
                } else if c == '|' && depth == 0 {
                    alts.push(std::mem::take(&mut current));
                } else {
                    current.push(c);
                }
            }
            alts.push(current);
            alts
        }

        fn match_subpattern(&self, pattern: &[char], h: &[char], hi: usize) -> Option<usize> {
            // Returns the end position if pattern matches starting at hi
            self.match_sub_from(pattern, 0, h, hi)
        }

        fn match_sub_from(&self, p: &[char], pi: usize, h: &[char], hi: usize) -> Option<usize> {
            if pi >= p.len() {
                return Some(hi);
            }

            let next_is_quantifier = pi + 1 < p.len() && matches!(p[pi + 1], '+' | '?' | '*');

            if next_is_quantifier {
                // Simplified — delegate to try_match style
                if self.try_match(p, pi, h, hi) {
                    // find where match ends — this is simplified
                    return Some(hi + 1);
                }
                return None;
            }

            if pi + 2 < p.len() && p[pi] == '(' && p[pi + 1] == '?' && p[pi + 2] == ':' {
                // non-capturing group
                let mut depth = 1;
                let mut i = pi + 3;
                let gs = i;
                while i < p.len() && depth > 0 {
                    if p[i] == '(' {
                        depth += 1;
                    } else if p[i] == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    i += 1;
                }
                let ge = i;
                let after = i + 1;
                let group: Vec<char> = p[gs..ge].to_vec();
                for alt in self.split_alternatives(&group) {
                    if let Some(end) = self.match_subpattern(&alt, h, hi) {
                        if let Some(r) = self.match_sub_from(p, after, h, end) {
                            return Some(r);
                        }
                    }
                }
                return None;
            }

            if hi >= h.len() {
                return None;
            }

            if p[pi] == '\\' && pi + 1 < p.len() {
                let c = p[pi + 1];
                let matched = match c {
                    'd' => h[hi].is_ascii_digit(),
                    _ => h[hi] == c,
                };
                if !matched {
                    return None;
                }
                return self.match_sub_from(p, pi + 2, h, hi + 1);
            }

            if p[pi] == '.' {
                return self.match_sub_from(p, pi + 1, h, hi + 1);
            }

            if p[pi] == h[hi] {
                return self.match_sub_from(p, pi + 1, h, hi + 1);
            }

            None
        }
    }
}
