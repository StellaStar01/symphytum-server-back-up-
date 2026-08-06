use std::collections::HashSet;

use types::rpc::api::common::UserData;

/// ids a gacha draw granted, so the response delta scopes to those rows.
#[derive(Default)]
pub struct GachaGranted {
    pub card_ids: HashSet<String>,
    pub costume_ids: HashSet<String>,
    pub sd_costume_ids: HashSet<String>,
    pub item_ids: HashSet<String>,
}

/// post-draw delta: granted rows + the drawn button.
pub fn after_gacha_draw(updated: &UserData, granted: &GachaGranted, button_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_card_list = updated
        .user_card_list
        .iter()
        .filter(|c| granted.card_ids.contains(&c.card_id))
        .cloned()
        .collect();
    d.user_costume_list = updated
        .user_costume_list
        .iter()
        .filter(|c| granted.costume_ids.contains(&c.costume_id))
        .cloned()
        .collect();
    d.user_sd_costume_list = updated
        .user_sd_costume_list
        .iter()
        .filter(|c| granted.sd_costume_ids.contains(&c.sd_costume_id))
        .cloned()
        .collect();
    d.user_item_list = updated
        .user_item_list
        .iter()
        .filter(|i| granted.item_ids.contains(&i.item_id))
        .cloned()
        .collect();
    d.user_gacha_button_list = updated
        .user_gacha_button_list
        .iter()
        .filter(|b| b.gacha_button_id == button_id)
        .cloned()
        .collect();
    d
}
