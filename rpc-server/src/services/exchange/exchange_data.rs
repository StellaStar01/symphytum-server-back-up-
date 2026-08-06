use resource::master::MasterTable;
use types::common::Reward;
use types::entity::master::{Card, ExchangeBooth, ExchangeBoothFixedItem, GachaPoint, Membership};
use types::enums::{CardRarity, ExchangeBoothItemType, ExchangeBoothType, ResourceType};
use types::rpc::api::exchange_item::Consumption;
use types::rpc::api::exchange_list_response::Booth;
use types::rpc::api::{ExchangeItem, ExchangeListResponse};

pub const GACHA_POINT_COST_PER_CARD: i64 = 200;

/// R5 cards by master order.
pub fn r5_cards_ordered() -> Vec<String> {
    let mut cards: Vec<&Card> = Card::table()
        .iter()
        .filter(|c| c.rarity == CardRarity::Rarity5 as i32)
        .collect();
    cards.sort_by_key(|c| (c.order, c.id.clone()));
    cards.into_iter().map(|c| c.id.clone()).collect()
}

/// (point_id, icon_asset_id) for an exchange booth group, from GachaPoint.
pub fn gacha_point_for_group(group_id: &str) -> Option<(&'static str, &'static str)> {
    GachaPoint::table()
        .iter()
        .find(|g| g.exchange_booth_group_id == group_id)
        .map(|g| (g.id.as_str(), g.icon_asset_id.as_str()))
}

/// booth id for a group: `exchange_booth-{group suffix}`.
fn booth_id_for_group(group_id: &str) -> Option<String> {
    group_id
        .strip_prefix("exchange_booth_group-")
        .map(|suffix| format!("exchange_booth-{suffix}"))
}

/// the group's booth rows (exact id match; masters keep one per gacha group).
pub fn group_booths(group_id: &str) -> Vec<&'static ExchangeBooth> {
    let Some(bid) = booth_id_for_group(group_id) else {
        return vec![];
    };
    ExchangeBooth::table()
        .iter()
        .filter(|b| b.id == bid)
        .collect()
}

/// fixed items for a booth, ordered by id.
pub fn group_items(booth_id: &str) -> Vec<&'static ExchangeBoothFixedItem> {
    let mut v: Vec<&ExchangeBoothFixedItem> = ExchangeBoothFixedItem::table()
        .iter()
        .filter(|i| i.exchange_booth_id == booth_id)
        .collect();
    v.sort_by_key(|i| i.id.clone());
    v
}

/// costs gacha_point * 200 and rewards the R5 card at the item's index (mod pool length).
pub fn gacha_item_response(
    item: &ExchangeBoothFixedItem,
    group_id: &str,
    purchased: i64,
) -> ExchangeItem {
    let (point_id, _icon) = gacha_point_for_group(group_id).unwrap_or(("", ""));
    let pool = r5_cards_ordered();
    let idx = group_items(&item.exchange_booth_id)
        .iter()
        .position(|i| i.id == item.id)
        .unwrap_or(0);
    let reward_id = pool
        .get(idx % pool.len().max(1))
        .cloned()
        .unwrap_or_default();
    ExchangeItem {
        booth_item_type: ExchangeBoothItemType::Fixed as i32,
        booth_item_id: item.id.clone(),
        name: String::new(),
        thumbnail_asset_id: String::new(),
        limit_quantity: if item.purchase_limit_quantity > 0 {
            item.purchase_limit_quantity
        } else {
            10
        },
        purchased_quantity: purchased as i32,
        consumptions: vec![Consumption {
            resource_type: ResourceType::GachaPoint as i32,
            resource_id: point_id.into(),
            quantity: GACHA_POINT_COST_PER_CARD,
            original_quantity: GACHA_POINT_COST_PER_CARD,
            discount_ratio_permil: 0,
        }],
        rewards: vec![Reward {
            resource_type: ResourceType::Card as i32,
            resource_id: reward_id,
            quantity: 1,
        }],
        next_reset_time: 0,
        end_time: 0,
        is_locked: false,
        unlock_condition_group_id: String::new(),
        description: String::new(),
        reset_interval_type: item.reset_interval_type,
        release_time: 0,
    }
}

/// booth id for a membership group: `exchange_booth-membership-chr-…`.
pub fn membership_booth_id(m: &Membership) -> String {
    let suffix = m
        .character_coin_exchange_booth_group_id
        .strip_prefix("exchange_booth_group-")
        .unwrap_or(&m.character_coin_exchange_booth_group_id);
    format!("exchange_booth-{suffix}")
}

/// the Membership row whose group id matches, if any.
pub fn membership_for_group(group_id: &str) -> Option<&'static Membership> {
    Membership::table()
        .iter()
        .find(|m| m.character_coin_exchange_booth_group_id == group_id)
}

/// synthesized emblem+stamp items for a membership row; masters have no booth rows for these.
pub fn membership_items(m: &Membership) -> Vec<ExchangeItem> {
    let booth_id = membership_booth_id(m);
    let mut items = Vec::with_capacity(m.emblem_ids.len() + m.stamp_ids.len());
    for (idx, eid) in m.emblem_ids.iter().enumerate() {
        items.push(membership_item(
            m,
            &booth_id,
            idx,
            eid,
            ResourceType::Emblem,
        ));
    }
    for (i, sid) in m.stamp_ids.iter().enumerate() {
        items.push(membership_item(
            m,
            &booth_id,
            m.emblem_ids.len() + i,
            sid,
            ResourceType::Stamp,
        ));
    }
    items
}

fn membership_item(
    m: &Membership,
    booth_id: &str,
    idx: usize,
    reward_id: &str,
    reward_type: ResourceType,
) -> ExchangeItem {
    ExchangeItem {
        booth_item_type: ExchangeBoothItemType::Fixed as i32,
        booth_item_id: format!("exchange_booth_fixed_item-{booth_id}-{idx}"),
        name: String::new(),
        thumbnail_asset_id: String::new(),
        limit_quantity: 1,
        purchased_quantity: 0,
        consumptions: vec![Consumption {
            resource_type: ResourceType::Item as i32,
            resource_id: m.character_coin_item_id.clone(),
            quantity: m.character_coin_item_quantity as i64,
            original_quantity: m.character_coin_item_quantity as i64,
            discount_ratio_permil: 0,
        }],
        rewards: vec![Reward {
            resource_type: reward_type as i32,
            resource_id: reward_id.into(),
            quantity: 1,
        }],
        next_reset_time: 0,
        end_time: 0,
        is_locked: false,
        unlock_condition_group_id: String::new(),
        description: String::new(),
        reset_interval_type: 0,
        release_time: 0,
    }
}

/// full list for a group: membership or gacha-point booth, else empty (marathon/facility have no masters).
pub fn group_response(group_id: &str) -> ExchangeListResponse {
    let mut resp = ExchangeListResponse {
        id: group_id.into(),
        name: group_id.into(),
        background_asset_id: "default-001".into(),
        header_asset_id: "default-001".into(),
        logo_asset_id: "default-001".into(),
        ..Default::default()
    };
    if let Some(m) = membership_for_group(group_id) {
        resp.booths = vec![Booth {
            id: membership_booth_id(m),
            name: String::new(),
            booth_type: ExchangeBoothType::Single as i32,
            items: membership_items(m),
            next_reset_time: 0,
            end_time: 0,
            is_locked: false,
            unlock_condition_group_id: String::new(),
        }];
        return resp;
    }
    if gacha_point_for_group(group_id).is_none() {
        return resp;
    }
    let Some(booth) = group_booths(group_id).into_iter().next() else {
        return resp;
    };
    resp.booths = vec![Booth {
        id: booth.id.clone(),
        name: String::new(),
        booth_type: ExchangeBoothType::Single as i32,
        items: group_items(&booth.id)
            .into_iter()
            .map(|i| gacha_item_response(i, group_id, 0))
            .collect(),
        next_reset_time: 0,
        end_time: 0,
        is_locked: false,
        unlock_condition_group_id: String::new(),
    }];
    resp
}

/// find a gacha-point item by id: (fixed item row, its group id).
pub fn gacha_item_group(item_id: &str) -> Option<(&'static ExchangeBoothFixedItem, String)> {
    ExchangeBoothFixedItem::table()
        .iter()
        .find(|i| i.id == item_id)
        .map(|i| {
            let group_id = i
                .exchange_booth_id
                .strip_prefix("exchange_booth-")
                .map(|suffix| format!("exchange_booth_group-{suffix}"))
                .unwrap_or_default();
            (i, group_id)
        })
}

/// find a membership item by id: (membership row, index into emblem+stamp list).
pub fn membership_item_by_id(item_id: &str) -> Option<(&'static Membership, usize)> {
    for m in Membership::table() {
        let booth_id = membership_booth_id(m);
        for idx in 0..(m.emblem_ids.len() + m.stamp_ids.len()) {
            if format!("exchange_booth_fixed_item-{booth_id}-{idx}") == item_id {
                return Some((m, idx));
            }
        }
    }
    None
}

/// the emblem/stamp resource for a membership item index.
pub fn membership_item_reward(m: &Membership, idx: usize) -> Option<(ResourceType, &str)> {
    if idx < m.emblem_ids.len() {
        Some((ResourceType::Emblem, m.emblem_ids[idx].as_str()))
    } else {
        let i = idx - m.emblem_ids.len();
        m.stamp_ids
            .get(i)
            .map(|s| (ResourceType::Stamp, s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use resource::master::MasterTable;
    use types::entity::master::ExchangeBoothGroup;

    async fn load_masters() {
        resource::master::load::<Card>().await.expect("card");
        resource::master::load::<ExchangeBooth>()
            .await
            .expect("booth");
        resource::master::load::<ExchangeBoothFixedItem>()
            .await
            .expect("fixed");
        resource::master::load::<ExchangeBoothGroup>()
            .await
            .expect("group");
        resource::master::load::<GachaPoint>().await.expect("point");
        resource::master::load::<Membership>()
            .await
            .expect("membership");
    }

    #[tokio::test]
    async fn gacha_group_builds_master_driven_items() {
        load_masters().await;
        let resp = group_response("exchange_booth_group-gacha-common-001");
        assert_eq!(resp.id, "exchange_booth_group-gacha-common-001");
        assert_eq!(resp.booths.len(), 1, "one booth per gacha group");
        let booth = &resp.booths[0];
        assert_eq!(booth.id, "exchange_booth-gacha-common-001");
        assert!(!booth.items.is_empty(), "gacha booth must have items");
        let item = &booth.items[0];
        assert_eq!(item.booth_item_type, 2, "FIXED");
        assert_eq!(item.consumptions.len(), 1);
        let c = &item.consumptions[0];
        assert_eq!(c.resource_type, 18, "GACHA_POINT");
        assert_eq!(c.resource_id, "gacha_point-common-001");
        assert_eq!(c.quantity, 200);
        assert_eq!(item.rewards.len(), 1);
        let pool = r5_cards_ordered();
        assert!(
            pool.contains(&item.rewards[0].resource_id),
            "reward must come from the R5 pool"
        );
        assert_eq!(item.rewards[0].resource_type, 2, "CARD");
    }

    #[tokio::test]
    async fn unknown_group_returns_empty_valid_response() {
        load_masters().await;
        let resp = group_response("exchange_booth_group-garbage");
        assert_eq!(resp.id, "exchange_booth_group-garbage");
        assert!(resp.booths.is_empty());
    }

    #[tokio::test]
    async fn membership_group_synthesizes_emblem_and_stamp_items() {
        load_masters().await;
        let resp = group_response("exchange_booth_group-membership-chr-00005");
        assert_eq!(resp.booths.len(), 1);
        let booth = &resp.booths[0];
        assert_eq!(booth.id, "exchange_booth-membership-chr-00005");
        assert!(!booth.items.is_empty());
        let item = &booth.items[0];
        assert_eq!(item.consumptions[0].resource_type, 3, "ITEM (coin)");
        assert!(
            item.consumptions[0]
                .resource_id
                .starts_with("item-character_coin-")
        );
        assert!(item.rewards[0].resource_id.starts_with("emblem-"));
    }
}
