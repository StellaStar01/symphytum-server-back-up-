use types::rpc::api::common::UserData;

/// post read-mark delta: the character's rows.
pub fn after_character_read(updated: &UserData, character_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_character_list = updated
        .user_character_list
        .iter()
        .filter(|c| c.character_id == character_id)
        .cloned()
        .collect();
    d
}
