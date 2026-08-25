//! Event system data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::EventID;
use crate::location::LocationRef;

/// Durable event metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDurable {
    pub aggregate_id: String,
    pub seq: i64,
    pub version: i64,
}

/// An event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub id: EventID,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub durable: Option<EventDurable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
