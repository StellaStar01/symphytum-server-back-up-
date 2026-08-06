use types::rpc::api::common::UserData;

/// post costume-mutation delta: the character's rows.
pub fn after_character(updated: &UserData, character_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_character_list = updated
        .user_character_list
        .iter()
        .filter(|c| c.character_id == character_id)
        .cloned()
        .collect();
    d
}

/// post costume-read delta: the costume rows whose read marker changed.
pub fn after_costume_read(
    updated: &UserData,
    costume_id: &str,
    sd_costume_id: &str,
    hair_accessory_id: &str,
) -> UserData {
    let mut d = UserData::default();
    d.user_costume_list = updated
        .user_costume_list
        .iter()
        .filter(|c| c.costume_id == costume_id)
        .cloned()
        .collect();
    d.user_sd_costume_list = updated
        .user_sd_costume_list
        .iter()
        .filter(|c| c.sd_costume_id == sd_costume_id)
        .cloned()
        .collect();
    d.user_sd_costume_hair_accessory_list = updated
        .user_sd_costume_hair_accessory_list
        .iter()
        .filter(|c| c.sd_costume_hair_accessory_id == hair_accessory_id)
        .cloned()
        .collect();
    d
}
