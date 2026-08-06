use database::Database;
use resource::master::MasterTable;
use tonic::Status;
use types::entity::master::{Card, CardPotential};
use types::enums::{GachaCardDuplicateRewardType, ResourceType};
use types::rpc::api::common::RewardResult;
use types::rpc::api::{GachaDrawResponseCardResult, GachaListResponseGachaButton};

use super::delta::GachaGranted;
use super::draw;

pub fn bonus_reward_results(
    button: &GachaListResponseGachaButton,
    before_quantities: &[i64],
) -> Vec<RewardResult> {
    button
        .bonus_rewards
        .iter()
        .zip(before_quantities)
        .map(|(r, before)| RewardResult {
            resource_type: r.resource_type,
            resource_id: r.resource_id.clone(),
            quantity: r.quantity,
            before_quantity: *before,
            after_quantity: before + r.quantity,
            is_new: false,
            is_gift: false,
        })
        .collect()
}

// the client always sends draw_count=1; the per-pull count lives on the button.
pub fn resolve_draw_count(
    button: &GachaListResponseGachaButton,
    draw_count: i32,
) -> Result<usize, Status> {
    let per_pull = button.total_reward_pick_count.max(1) as usize;
    let count = (draw_count.max(1) as usize).saturating_mul(per_pull);
    if count > 100 {
        return Err(Status::invalid_argument("too many draws"));
    }
    Ok(count)
}

/// consume the button's costs; free stones are spent first (the split is inside consume_stones).
pub async fn consume_costs(
    db: &Database,
    uid: &str,
    button: &GachaListResponseGachaButton,
) -> Result<(), Status> {
    let mut free_cost = 0i64;
    let mut paid_cost = 0i64;
    let mut item_costs: Vec<(String, i64)> = Vec::new();
    for c in &button.consumptions {
        match ResourceType::try_from(c.resource_type).unwrap_or(ResourceType::Unknown) {
            ResourceType::StoneTotal => free_cost += c.quantity,
            ResourceType::StonePaidOnly => paid_cost += c.quantity,
            ResourceType::Item => item_costs.push((c.resource_id.clone(), c.quantity)),
            _ => {}
        }
    }
    if free_cost + paid_cost > 0 {
        let ok = db
            .consume_stones(uid, free_cost, paid_cost)
            .await
            .map_err(|e| Status::internal(format!("consume stones: {e}")))?;
        if !ok {
            return Err(Status::failed_precondition("insufficient diamonds"));
        }
    }
    for (item_id, qty) in &item_costs {
        let ok = db
            .consume_item(uid, item_id, *qty)
            .await
            .map_err(|e| Status::internal(format!("consume item: {e}")))?;
        if !ok {
            return Err(Status::failed_precondition("insufficient item"));
        }
    }
    Ok(())
}

/// grant the draw results, bloom-aware; returns results + granted ids for the delta.
pub async fn grant_results(
    db: &Database,
    uid: &str,
    results: Vec<GachaDrawResponseCardResult>,
) -> Result<(Vec<GachaDrawResponseCardResult>, GachaGranted), Status> {
    let now = database::unix_now();
    let mut out = Vec::with_capacity(results.len());
    let mut granted = GachaGranted::default();
    for r in results {
        let Some(card) = Card::table().iter().find(|c| c.id == r.card_id) else {
            continue;
        };
        granted.card_ids.insert(card.id.clone());
        let (is_new, existing_potential) = db
            .grant_card(uid, &r.card_id, now)
            .await
            .map_err(|e| Status::internal(format!("grant card: {e}")))?;

        if is_new {
            if !card.reward_costume_id.is_empty() {
                db.grant_costume(uid, &card.reward_costume_id, now)
                    .await
                    .map_err(|e| Status::internal(format!("grant costume: {e}")))?;
                granted.costume_ids.insert(card.reward_costume_id.clone());
            }
            if !card.reward_sd_costume_id.is_empty() {
                db.grant_sd_costume(uid, &card.reward_sd_costume_id, now)
                    .await
                    .map_err(|e| Status::internal(format!("grant sd costume: {e}")))?;
                granted
                    .sd_costume_ids
                    .insert(card.reward_sd_costume_id.clone());
            }
            if let Some(acq) = card.acquire_reward.as_ref() {
                db.add_item(uid, &acq.resource_id, acq.quantity, now)
                    .await
                    .map_err(|e| Status::internal(format!("grant acquire: {e}")))?;
                granted.item_ids.insert(acq.resource_id.clone());
            }
            out.push(draw::build_card_result(card));
        } else {
            // duplicate: bloom-aware compensation
            let max_potential = CardPotential::table()
                .iter()
                .filter(|p| p.group_id == card.card_potential_group_id)
                .map(|p| p.upgrade_count)
                .max()
                .unwrap_or(0);
            let mut card_result = GachaDrawResponseCardResult {
                card_id: card.id.clone(),
                duplicate_reward_type: GachaCardDuplicateRewardType::None as i32,
                ..Default::default()
            };
            if existing_potential < max_potential {
                // not max bloom: convert to potential upgrade points
                card_result.duplicate_reward_type =
                    GachaCardDuplicateRewardType::PotentialUpgradePoint as i32;
                card_result.provided_potential_upgrade_point_quantity = 1;
                card_result.card_reward_results.push(RewardResult {
                    resource_type: ResourceType::Card as i32,
                    resource_id: card.id.clone(),
                    quantity: 1,
                    before_quantity: 1,
                    after_quantity: 1,
                    is_new: false,
                    is_gift: false,
                });
            } else {
                // max bloom: grant the card's alternative compensation (sticker)
                card_result.duplicate_reward_type =
                    GachaCardDuplicateRewardType::PotentialAlternativeReward as i32;
                let mut rewards = vec![RewardResult {
                    resource_type: ResourceType::Card as i32,
                    resource_id: card.id.clone(),
                    quantity: 1,
                    before_quantity: 1,
                    after_quantity: 1,
                    is_new: false,
                    is_gift: false,
                }];
                if let Some(alt) = card.potential_alternative_reward.as_ref() {
                    let before = db
                        .item_quantity(uid, &alt.resource_id)
                        .await
                        .map_err(|e| Status::internal(format!("alt qty: {e}")))?;
                    db.add_item(uid, &alt.resource_id, alt.quantity, now)
                        .await
                        .map_err(|e| Status::internal(format!("grant alt: {e}")))?;
                    granted.item_ids.insert(alt.resource_id.clone());
                    rewards.push(RewardResult {
                        resource_type: alt.resource_type,
                        resource_id: alt.resource_id.clone(),
                        quantity: alt.quantity,
                        before_quantity: before,
                        after_quantity: before + alt.quantity,
                        is_new: false,
                        is_gift: false,
                    });
                }
                card_result.card_reward_results = rewards;
            }
            out.push(card_result);
        }
    }
    Ok((out, granted))
}
