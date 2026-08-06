use types::rpc::api::common::UserData;

/// post skill-tree mutation delta: the character's skill tree rows.
pub fn after_skill_tree(updated: &UserData, character_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_character_skill_tree_list = updated
        .user_character_skill_tree_list
        .iter()
        .filter(|s| s.character_id == character_id)
        .cloned()
        .collect();
    d
}
