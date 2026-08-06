use types::rpc::api::common::UserData;

/// post-save-deck delta: the deck + its positions.
pub fn after_save_deck(updated: &UserData, character_id: &str, number: i32) -> UserData {
    let mut d = UserData::default();
    d.user_live_deck_list = updated
        .user_live_deck_list
        .iter()
        .filter(|x| x.character_id == character_id && x.number == number)
        .cloned()
        .collect();
    d.user_live_deck_position_list = updated
        .user_live_deck_position_list
        .iter()
        .filter(|x| x.character_id == character_id && x.number == number)
        .cloned()
        .collect();
    d
}

/// post-live-finish delta: the played music's rows + live stamina state.
pub fn after_finish_live(updated: &UserData, music_id: &str, character_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_music_list = updated
        .user_music_list
        .iter()
        .filter(|m| m.music_id == music_id)
        .cloned()
        .collect();
    d.user_music_difficulty_list = updated
        .user_music_difficulty_list
        .iter()
        .filter(|m| m.music_id == music_id)
        .cloned()
        .collect();
    d.user_music_character_highest_score_list = updated
        .user_music_character_highest_score_list
        .iter()
        .filter(|s| s.music_id == music_id && s.character_id == character_id)
        .cloned()
        .collect();
    d.user_live = updated.user_live.clone();
    d
}
