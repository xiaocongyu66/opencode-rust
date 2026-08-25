//! HTTP middleware for the opencode protocol.
//!
//! - [`authorization`] — request authorization gate.
//! - [`schema_error`] — schema validation error normalization.

pub mod authorization;
pub mod schema_error;
