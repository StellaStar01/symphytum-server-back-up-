use database::Database;
use resource::master::MasterTable;
use types::entity::master::{JumpRope, JumpRopeJumpCountReward, JumpRopeSetting};
use types::rpc::api::common::RewardResult;

/// highest reward tier at or below the jump count; quantities are the jump count itself (in no master).
pub fn tier_rewards(rope: &JumpRope, jump_count: i32) -> Vec<&'static JumpRopeJumpCountReward> {
    if rope.jump_rope_jump_count_reward_group_id.is_empty() {
        return vec![];
    }
    JumpRopeJumpCountReward::table()
        .iter()
        .filter(|r| {
            r.group_id == rope.jump_rope_jump_count_reward_group_id
                && r.jump_count > 0
                && r.jump_count <= jump_count
        })
        .max_by_key(|r| r.jump_count)
        .into_iter()
        .collect()
}

/// grant tier rewards; each quantity is the jump count (masters carry none).
pub async fn grant_tier_rewards(
    db: &Database,
    uid: &str,
    rope: &JumpRope,
    jump_count: i32,
    now: i64,
) -> Result<Vec<RewardResult>, sqlx::Error> {
    let mut results = Vec::new();
    for tier in tier_rewards(rope, jump_count) {
        for rw in &tier.rewards {
            let before = db.item_quantity(uid, &rw.resource_id).await?;
            db.add_item(uid, &rw.resource_id, jump_count as i64, now)
                .await?;
            results.push(RewardResult {
                resource_type: rw.resource_type,
                resource_id: rw.resource_id.clone(),
                quantity: jump_count as i64,
                before_quantity: before,
                after_quantity: before + jump_count as i64,
                is_new: false,
                is_gift: false,
            });
        }
    }
    Ok(results)
}

/// reward-up stamina unit from JumpRopeSetting row 0 (15 matches the capture).
pub fn reward_up_stamina_consumption_quantity() -> i32 {
    JumpRopeSetting::table()
        .first()
        .map(|s| s.reward_up_stamina_consumption_unit)
        .unwrap_or(15)
}
