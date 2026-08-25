//! Common types and ID generation utilities.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use rand::Rng;

/// A relative filesystem path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct RelativePath(pub String);

impl RelativePath {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for RelativePath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// An absolute filesystem path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct AbsolutePath(pub String);

impl AbsolutePath {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for AbsolutePath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

static LAST_TIMESTAMP: AtomicI64 = AtomicI64::new(0);
static COUNTER: AtomicU64 = AtomicU64::new(0);

const ID_LENGTH: usize = 26;
const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Generate an ascending-sortable ULID-style ID (time-prefixed).
pub fn ascending() -> String {
    create_id(false)
}

/// Generate a descending-sortable ULID-style ID (reverse time-prefixed).
pub fn descending() -> String {
    create_id(true)
}

fn create_id(descending: bool) -> String {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let prev = LAST_TIMESTAMP.load(Ordering::Relaxed);
    let counter = if timestamp != prev {
        LAST_TIMESTAMP.store(timestamp, Ordering::Relaxed);
        COUNTER.store(1, Ordering::Relaxed);
        1u64
    } else {
        COUNTER.fetch_add(1, Ordering::Relaxed) + 1
    };

    let current = (timestamp as u64).wrapping_mul(0x1000).wrapping_add(counter);
    let value = if descending {
        !current
    } else {
        current
    };

    let mut time_part = String::with_capacity(12);
    for i in 0..6usize {
        let byte = ((value >> (40 - 8 * i)) & 0xff) as u8;
        time_part.push_str(&format!("{:02x}", byte));
    }

    let mut rng = rand::thread_rng();
    let mut random_part = String::with_capacity(ID_LENGTH - 12);
    for _ in 0..(ID_LENGTH - 12) {
        random_part.push(CHARS[rng.gen_range(0..CHARS.len())] as char);
    }

    format!("{}{}", time_part, random_part)
}
