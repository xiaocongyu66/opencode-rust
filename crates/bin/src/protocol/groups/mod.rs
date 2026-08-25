//! API route group definitions.
//!
//! Each module defines route path constants and an [`ApiGroup`](crate::protocol::api::ApiGroup)
//! implementation for one logical group of endpoints, mirroring the TypeScript
//! `HttpApiGroup` definitions in `packages/protocol/src/groups/`.

pub mod agent;
pub mod command;
pub mod credential;
pub mod event;
pub mod fs;
pub mod health;
pub mod integration;
pub mod location;
pub mod message;
pub mod model;
pub mod permission;
pub mod project_copy;
pub mod provider;
pub mod pty;
pub mod question;
pub mod reference;
pub mod session;
pub mod skill;
