use database::Database;
use tonic::Status;
use types::enums::ResourceType;
use types::rpc::api::common::{Response as CommonResponse, RewardResult};
use types::rpc::api::{ExchangePurchaseRequest, ExchangePurchaseResponse};

use super::delta;
use super::exchange_data;

pub async fn purchase(
    db: &Database,
    uid: &str,
    req: ExchangePurchaseRequest,
) -> Result<ExchangePurchaseResponse, Status> {
    let now = database::unix_now();
    let qty = req.quantity.max(1) as i64;

    // --- gacha-point items (masters: ExchangeBoothFixedItem + GachaPoint) ---
    if let Some((item, group_id)) = exchange_data::gacha_item_group(&req.booth_item_id) {
        let booth_id = item.exchange_booth_id.clone();
        let (point_id, _) = exchange_data::gacha_point_for_group(&group_id)
            .ok_or_else(|| Status::invalid_argument("no gacha point for group"))?;
        let limit = if item.purchase_limit_quantity > 0 {
            item.purchase_limit_quantity as i64
        } else {
            10
        };
        let purchased = db
            .exchange_purchase_count(uid, &booth_id, &req.booth_item_id)
            .await
            .map_err(|e| Status::internal(format!("purchase count: {e}")))?;
        if purchased + qty > limit {
            return Err(Status::failed_precondition("purchase limit reached"));
        }

        // the reward card is the one the list response advertises
        let pool = exchange_data::r5_cards_ordered();
        let idx = exchange_data::group_items(&booth_id)
            .iter()
            .position(|i| i.id == req.booth_item_id)
            .unwrap_or(0);
        let card_id = pool
            .get(idx % pool.len().max(1))
            .cloned()
            .ok_or_else(|| Status::invalid_argument("no reward card"))?;

        // consume gacha points, grant the card (cards are items here)
        let cost = exchange_data::GACHA_POINT_COST_PER_CARD * qty;
        let before_point = db
            .item_quantity(uid, point_id)
            .await
            .map_err(|e| Status::internal(format!("point qty: {e}")))?;
        if before_point < cost {
            return Err(Status::failed_precondition("insufficient gacha points"));
        }
        let ok = db
            .consume_item(uid, point_id, cost)
            .await
            .map_err(|e| Status::internal(format!("consume points: {e}")))?;
        if !ok {
            return Err(Status::failed_precondition("insufficient gacha points"));
        }
        let before_card = db
            .item_quantity(uid, &card_id)
            .await
            .map_err(|e| Status::internal(format!("card qty: {e}")))?;
        db.add_item(uid, &card_id, qty, now)
            .await
            .map_err(|e| Status::internal(format!("grant card: {e}")))?;
        db.bump_exchange_purchase(uid, &booth_id, &req.booth_item_id, qty, now)
            .await
            .map_err(|e| Status::internal(format!("record purchase: {e}")))?;

        let updated = db
            .user_data(uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        return Ok(ExchangePurchaseResponse {
            reward_results: vec![RewardResult {
                resource_type: ResourceType::Item as i32,
                resource_id: card_id,
                quantity: qty,
                before_quantity: before_card,
                after_quantity: before_card + qty,
                is_new: false,
                is_gift: false,
            }],
            after_item: Some(exchange_data::gacha_item_response(
                item,
                &group_id,
                purchased + qty,
            )),
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_exchange_booth_read(&updated, &booth_id)),
                ..Default::default()
            }),
        });
    }

    // --- membership items (synthesized emblem/stamp exchange) ---
    if let Some((m, idx)) = exchange_data::membership_item_by_id(&req.booth_item_id) {
        let booth_id = exchange_data::membership_booth_id(m);
        let (reward_type, reward_id) = exchange_data::membership_item_reward(m, idx)
            .ok_or_else(|| Status::invalid_argument("unknown membership item"))?;
        let limit = 1i64;
        let purchased = db
            .exchange_purchase_count(uid, &booth_id, &req.booth_item_id)
            .await
            .map_err(|e| Status::internal(format!("purchase count: {e}")))?;
        if purchased + qty > limit {
            return Err(Status::failed_precondition("purchase limit reached"));
        }

        let coin_id = m.character_coin_item_id.clone();
        let cost = m.character_coin_item_quantity as i64 * qty;
        let ok = db
            .consume_item(uid, &coin_id, cost)
            .await
            .map_err(|e| Status::internal(format!("consume coins: {e}")))?;
        if !ok {
            return Err(Status::failed_precondition("insufficient character coins"));
        }
        let before = db
            .item_quantity(uid, reward_id)
            .await
            .map_err(|e| Status::internal(format!("reward qty: {e}")))?;
        db.add_item(uid, reward_id, qty, now)
            .await
            .map_err(|e| Status::internal(format!("grant reward: {e}")))?;
        db.bump_exchange_purchase(uid, &booth_id, &req.booth_item_id, qty, now)
            .await
            .map_err(|e| Status::internal(format!("record purchase: {e}")))?;

        let updated = db
            .user_data(uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        return Ok(ExchangePurchaseResponse {
            reward_results: vec![RewardResult {
                resource_type: reward_type as i32,
                resource_id: reward_id.into(),
                quantity: qty,
                before_quantity: before,
                after_quantity: before + qty,
                is_new: false,
                is_gift: false,
            }],
            after_item: Some(
                exchange_data::membership_items(m)
                    .into_iter()
                    .find(|i| i.booth_item_id == req.booth_item_id)
                    .map(|mut i| {
                        i.purchased_quantity = (purchased + qty) as i32;
                        i
                    })
                    .unwrap_or_default(),
            ),
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_exchange_booth_read(&updated, &booth_id)),
                ..Default::default()
            }),
        });
    }

    Err(Status::invalid_argument("unknown booth_item_id"))
}
