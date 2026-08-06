use types::rpc::api::common::UserData;

/// post jump-rope delta: the rope's row (best / cleared / play count / npc exits).
pub fn after_jump_rope(updated: &UserData, rope_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_jump_rope_list = updated
        .user_jump_rope_list
        .iter()
        .filter(|r| r.jump_rope_id == rope_id)
        .cloned()
        .collect();
    d
}
