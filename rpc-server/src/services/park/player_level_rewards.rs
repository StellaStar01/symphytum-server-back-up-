use std::collections::HashSet;

use database::Database;
use resource::master::MasterTable;
use tonic::Status;
use types::entity::master::PlayerLevel;
use types::enums::ResourceType;
use types::rpc::api::common::RewardResult;

/// grant unclaimed player-level rewards, bloom-aware; returns results + the full received-level list.
pub async fn player_level_rewards(
    db: &Database,
    uid: &str,
    player_levels: &[i32],
) -> Result<(Vec<RewardResult>, Vec<i32>), Status> {
    let levels = PlayerLevel::table();

    let claimed: HashSet<i32> = db
        .park_received_levels(uid)
        .await
        .map_err(|e| Status::internal(format!("park rewards: {e}")))?
        .into_iter()
        .collect();
    let now = database::unix_now();
    let mut reward_results = Vec::new();
    let mut newly = Vec::new();
    for lv in player_levels {
        if claimed.contains(lv) {
            continue;
        }
        let Some(pl) = levels.iter().find(|p| p.level == *lv) else {
            continue;
        };
        for r in &pl.rewards {
            let (before, after) =
                match ResourceType::try_from(r.resource_type).unwrap_or(ResourceType::Unknown) {
                    ResourceType::StoneTotal => {
                        let (free, _) = db
                            .balances(uid)
                            .await
                            .map_err(|e| Status::internal(format!("balance: {e}")))?;
                        let before = free as i64;
                        db.grant_stones(uid, r.quantity, 0)
                            .await
                            .map_err(|e| Status::internal(format!("grant stones: {e}")))?;
                        (before, before + r.quantity)
                    }
                    ResourceType::Item => {
                        let before = db
                            .item_quantity(uid, &r.resource_id)
                            .await
                            .map_err(|e| Status::internal(format!("item qty: {e}")))?;
                        db.add_item(uid, &r.resource_id, r.quantity, now)
                            .await
                            .map_err(|e| Status::internal(format!("grant item: {e}")))?;
                        (before, before + r.quantity)
                    }
                    _ => continue,
                };
            reward_results.push(RewardResult {
                resource_type: r.resource_type,
                resource_id: r.resource_id.clone(),
                quantity: r.quantity,
                before_quantity: before,
                after_quantity: after,
                is_new: false,
                is_gift: false,
            });
        }
        db.add_park_level_reward(uid, *lv)
            .await
            .map_err(|e| Status::internal(format!("record reward: {e}")))?;
        newly.push(*lv);
    }

    let mut received: Vec<i32> = claimed.into_iter().collect();
    received.extend(newly);
    received.sort_unstable();
    Ok((reward_results, received))
}
