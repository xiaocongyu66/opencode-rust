//! Skill routes — `packages/protocol/src/groups/skill.ts`

use axum::Router;

use crate::api::ApiGroup;

/// `GET` — List currently registered skills.
pub const SKILL_LIST: &str = "/api/skill";

/// Skill API group.
pub struct SkillGroup;

impl ApiGroup for SkillGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
