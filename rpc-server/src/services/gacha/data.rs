use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;

use resource::master::MasterTable;
use types::common::{Consumption, Reward};
use types::entity::master::{Gacha, GachaButton, GachaPoint};
use types::enums::{GachaType, ResourceType};
use types::rpc::api::{
    GachaListResponse, GachaListResponseCardSelect, GachaListResponseGacha,
    GachaListResponseGachaButton, GachaListResponseGachaGroup, GachaListResponseGachaPoint,
};

// gacha_id -> display name. Add new banners here as master rows appear.
pub static GACHA_NAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("gacha-common-normal-001", "holodori Gacha"),
        ("gacha-common-ticket_r5-001", "R5 Ticket Gacha"),
        ("gacha-common-ticket_r4-001", "R4 Ticket Gacha"),
        ("gacha-beginner-select-001", "Beginner's Support Gacha"),
        ("gacha-fixed-beginner-001", "Beginner Fixed Gacha"),
        ("gacha-pickup-normal-260728", "Sunny Summer Vacay Gacha"),
        (
            "gacha-pickup-select-260728",
            "Take your pick! Sunny Summer Vacay Gacha",
        ),
        ("gacha-fixed-260807", "Sunny Summer Vacay Fixed Gacha"),
        ("gacha-pickup-normal-260807", "Sunny Summer Vacay Gacha"),
    ])
});

// gacha_id -> card_ids chosen via SetSelectedCard
pub static SELECTED_CARDS: LazyLock<Mutex<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// gacha point for a gacha: gacha_id minus its -normal/-select/-fixed segment; tickets have no row.
pub fn gacha_point_for_gacha(gacha_id: &str) -> Option<(&'static str, &'static str)> {
    let stripped = gacha_id
        .replace("-normal", "")
        .replace("-select", "")
        .replace("-fixed", "");
    let group = format!("exchange_booth_group-{stripped}");
    GachaPoint::table()
        .iter()
        .find(|g| g.exchange_booth_group_id == group)
        .map(|g| (g.id.as_str(), g.icon_asset_id.as_str()))
}

/// pulls per button press from the button-id convention (btn_10- = 10).
fn pulls_for_button(button_id: &str) -> i32 {
    if button_id.contains("btn_10-") { 10 } else { 1 }
}

/// per-pull cost from the button-id convention: -stone = 250, -paid_stone = 90, -gacha_ticket/-gacha-ticket = tickets.
fn cost_for_button(button_id: &str) -> (ResourceType, &'static str, i64) {
    if button_id.contains("-paid_stone") {
        (ResourceType::StonePaidOnly, "", 90)
    } else if button_id.contains("-stone") {
        (ResourceType::StoneTotal, "", 250)
    } else if button_id.contains("-gacha_ticket") {
        (ResourceType::Item, "item-gacha_ticket-cmn-01", 1)
    } else if button_id.contains("-gacha-ticket") {
        (ResourceType::Item, "item-gacha_ticket-r5-01", 1)
    } else {
        (ResourceType::Unknown, "", 0)
    }
}

fn button_response(b: &GachaButton) -> GachaListResponseGachaButton {
    let pulls = pulls_for_button(&b.id);
    let (cost_type, cost_id, per_pull) = cost_for_button(&b.id);
    let consumptions = if cost_type == ResourceType::Unknown {
        vec![]
    } else {
        vec![Consumption {
            resource_type: cost_type as i32,
            resource_id: cost_id.into(),
            quantity: per_pull * pulls as i64,
        }]
    };
    let bonus_rewards = gacha_point_for_gacha(&b.gacha_id)
        .map(|(pid, _)| {
            vec![Reward {
                resource_type: ResourceType::GachaPoint as i32,
                resource_id: pid.into(),
                quantity: pulls as i64,
            }]
        })
        .unwrap_or_default();
    GachaListResponseGachaButton {
        gacha_button_id: b.id.clone(),
        name: String::new(),
        description: String::new(),
        is_disabled: false,
        consumptions,
        bonus_rewards,
        limit_count: 0,
        reset_interval_type: b.reset_interval_type,
        total_reward_pick_count: pulls,
        fixed_reward_pick_count: 0,
        drawn_count: 0,
        priority: 0,
    }
}

fn gacha_response(g: &Gacha) -> GachaListResponseGacha {
    let is_card_select = g.id.contains("-select-");
    let gacha_type =
        if g.id.contains("-normal-") || g.id.contains("-common-") || g.id.contains("-fixed-") {
            GachaType::Normal
        } else {
            GachaType::CardSelect
        };
    let gacha_point = gacha_point_for_gacha(&g.id).map(|(pid, _)| GachaListResponseGachaPoint {
        gacha_point_id: pid.into(),
        quantity: 1,
    });
    let card_select = if is_card_select {
        Some(GachaListResponseCardSelect {
            selectable_card_ids: vec![],
            selectable_card_quantity: if g.id == "gacha-beginner-select-001" {
                3
            } else {
                1
            },
            selected_card_ids: vec![],
        })
    } else {
        None
    };
    let buttons: Vec<GachaListResponseGachaButton> = GachaButton::table()
        .iter()
        .filter(|b| b.gacha_id == g.id)
        .map(button_response)
        .collect();
    GachaListResponseGacha {
        gacha_id: g.id.clone(),
        r#type: gacha_type as i32,
        name: GACHA_NAMES
            .get(g.id.as_str())
            .copied()
            .unwrap_or(g.id.as_str())
            .to_string(),
        is_locked: false,
        unlock_condition_group_id: String::new(),
        start_time: 0,
        end_time: 0,
        detail_notice_id: String::new(),
        precaution: String::new(),
        icon_asset_id: String::new(),
        pickup_card_ids: vec![],
        promotion_pickup_card_ids: vec![],
        promotion_movie_asset_id: String::new(),
        promotion_image_asset_id: String::new(),
        is_display_promotion_pickup_card_random: false,
        gacha_buttons: buttons,
        gacha_point,
        card_bonuses: vec![],
        card_select,
        is_end_time_hidden: false,
        gacha_animation_grouping_id: "gacha_animation_grouping-default-001".into(),
        gacha_animation_movie_group_id: String::new(),
        gacha_animation_asset_id: String::new(),
        read_time: 0,
        bgm_asset_id: String::new(),
        is_promotion_pickup_card_movie: false,
        fixed_slot_name: String::new(),
        promotion_text: String::new(),
        promotion_sub_text: String::new(),
        term_limited_hours: 0,
    }
}

/// gacha list from the Gacha + GachaButton masters, one group per banner.
pub fn list() -> GachaListResponse {
    GachaListResponse {
        gacha_groups: Gacha::table()
            .iter()
            .map(|g| GachaListResponseGachaGroup {
                gacha_group_id: format!("gacha_group-{}", g.id),
                icon_asset_id: String::new(),
                gachas: vec![gacha_response(g)],
            })
            .collect(),
        common_response: None,
    }
}

// (gacha_id, button) for a button id present in the list
pub fn find_button(button_id: &str) -> Option<(String, GachaListResponseGachaButton)> {
    list()
        .gacha_groups
        .into_iter()
        .flat_map(|g| g.gachas.into_iter())
        .find_map(|gacha| {
            let button = gacha
                .gacha_buttons
                .into_iter()
                .find(|b| b.gacha_button_id == button_id)?;
            Some((gacha.gacha_id, button))
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

    #[tokio::test]
    async fn list_builds_from_masters() {
        resource::master::load::<Gacha>().await.expect("gacha");
        resource::master::load::<GachaButton>()
            .await
            .expect("buttons");
        resource::master::load::<GachaPoint>()
            .await
            .expect("points");
        let l = list();
        assert!(!l.gacha_groups.is_empty(), "at least one banner");
        let total_buttons: usize = l
            .gacha_groups
            .iter()
            .flat_map(|g| g.gachas.iter())
            .map(|g| g.gacha_buttons.len())
            .sum();
        assert_eq!(
            total_buttons,
            GachaButton::table().len(),
            "all master buttons in list"
        );
    }

    #[tokio::test]
    async fn find_button_resolves_all_master_buttons() {
        resource::master::load::<Gacha>().await.expect("gacha");
        resource::master::load::<GachaButton>()
            .await
            .expect("buttons");
        resource::master::load::<GachaPoint>()
            .await
            .expect("points");
        let l = list();
        let mut count = 0;
        for gacha in l.gacha_groups.iter().flat_map(|g| g.gachas.iter()) {
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

    #[tokio::test]
    async fn ten_pull_button_costs_ten_times() {
        resource::master::load::<Gacha>().await.expect("gacha");
        resource::master::load::<GachaButton>()
            .await
            .expect("buttons");
        resource::master::load::<GachaPoint>()
            .await
            .expect("points");
        let (_, ten) = find_button("gacha_button-common-normal-001-btn_10-stone")
            .expect("btn_10-stone in list");
        assert_eq!(ten.total_reward_pick_count, 10);
        assert_eq!(ten.consumptions.len(), 1);
        assert_eq!(ten.consumptions[0].quantity, 2500, "250 * 10 pulls");
        assert_eq!(
            ten.consumptions[0].resource_type,
            ResourceType::StoneTotal as i32
        );
        // gacha-common-normal-001 has a gacha point bonus
        assert_eq!(ten.bonus_rewards.len(), 1);
        assert_eq!(ten.bonus_rewards[0].resource_id, "gacha_point-common-001");
        assert_eq!(ten.bonus_rewards[0].quantity, 10);

        let (_, paid) = find_button("gacha_button-common-normal-001-btn_01-paid_stone-daily")
            .expect("paid daily in list");
        assert_eq!(paid.consumptions[0].quantity, 90);
        assert_eq!(
            paid.consumptions[0].resource_type,
            ResourceType::StonePaidOnly as i32
        );

        let (_, ticket) =
            find_button("gacha_button-gacha_common-ticket_r5-001-btn_01-gacha-ticket")
                .expect("r5 ticket in list");
        assert_eq!(
            ticket.consumptions[0].resource_id, "item-gacha_ticket-r5-01",
            "r5 ticket button uses the r5 ticket item"
        );
    }
}
