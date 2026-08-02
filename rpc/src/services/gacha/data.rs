use std::collections::HashMap;
use std::sync::LazyLock;
use parking_lot::Mutex;

use prost::Message;

use resource::master::MasterTable;
use types::entity::master::GachaButton;
use types::rpc::api::{GachaListResponse, GachaListResponseGacha, GachaListResponseGachaButton};

use crate::sniffs;

pub static LIST: LazyLock<GachaListResponse> = LazyLock::new(|| {
    GachaListResponse::decode(sniffs::GACHA_LIST_RESP).expect("GACHA_LIST_RESP must decode")
});

// gacha_id -> card_ids chosen via SetSelectedCard
pub static SELECTED_CARDS: LazyLock<Mutex<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// (gacha, button) for a button id present in the List capture
pub fn find_button<'a>(
    button_id: &str,
) -> Option<(&'a GachaListResponseGacha, &'a GachaListResponseGachaButton)> {
    LIST.gacha_groups
        .iter()
        .flat_map(|g| g.gachas.iter())
        .find_map(|gacha| {
            gacha
                .gacha_buttons
                .iter()
                .find(|b| b.gacha_button_id == button_id)
                .map(|b| (gacha, b))
        })
}

// gacha_id for a button id, from the master GachaButton table
pub fn gacha_id_of_button(button_id: &str) -> Option<&'static str> {
    GachaButton::table()
        .iter()
        .find(|b| b.id == button_id)
        .map(|b| b.gacha_id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_button_resolves_all_sniff_buttons() {
        let mut count = 0;
        for gacha in LIST.gacha_groups.iter().flat_map(|g| g.gachas.iter()) {
            for button in &gacha.gacha_buttons {
                let found = find_button(&button.gacha_button_id);
                assert!(
                    found.is_some(),
                    "button {} not found",
                    button.gacha_button_id
                );
                count += 1;
            }
        }
        assert!(count > 0, "no buttons in list");
        assert!(find_button("garbage").is_none());
    }
}
