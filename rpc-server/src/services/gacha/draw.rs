use rand::Rng;
use rand::RngExt;

use resource::master::MasterTable;
use types::entity::master::Card;
use types::enums::{CardRarity, GachaCardDuplicateRewardType, ResourceType};
use types::rpc::api::GachaDrawResponseCardResult;
use types::rpc::api::common::RewardResult;

use super::rates::RARITY_WEIGHTS;

pub fn roll_tier(rng: &mut impl Rng, tiers: &[(CardRarity, u64)]) -> CardRarity {
    let parts: Vec<u64> = tiers
        .iter()
        .map(|(r, _)| super::rates::tier_parts(*r, tiers))
        .collect();
    let roll = rng.random_range(0..100_000_000u64);
    let mut acc = 0;
    for ((r, _), p) in tiers.iter().zip(parts) {
        acc += p;
        if roll < acc {
            return *r;
        }
    }
    tiers.last().unwrap().0
}

fn roll_card<'a>(rng: &mut impl Rng, pool: &'a [&'a Card], rarity: CardRarity) -> &'a Card {
    let tier_cards = super::rates::cards_of_rarity(pool, rarity);
    tier_cards[rng.random_range(0..tier_cards.len())]
}

fn reward_result(resource_type: ResourceType, resource_id: String, quantity: i64) -> RewardResult {
    RewardResult {
        resource_type: resource_type as i32,
        resource_id,
        quantity,
        before_quantity: 0,
        after_quantity: quantity,
        is_new: true,
        is_gift: false,
    }
}

// reward order must match the real server: card, costume, sd costume, acquire_reward.
pub fn build_card_result(card: &Card) -> GachaDrawResponseCardResult {
    let mut rewards = vec![reward_result(ResourceType::Card, card.id.clone(), 1)];
    if !card.reward_costume_id.is_empty() {
        rewards.push(reward_result(
            ResourceType::Costume,
            card.reward_costume_id.clone(),
            1,
        ));
    }
    if !card.reward_sd_costume_id.is_empty() {
        rewards.push(reward_result(
            ResourceType::SdCostume,
            card.reward_sd_costume_id.clone(),
            1,
        ));
    }
    if let Some(acq) = card.acquire_reward.as_ref() {
        rewards.push(reward_result(
            ResourceType::try_from(acq.resource_type).unwrap_or(ResourceType::Unknown),
            acq.resource_id.clone(),
            acq.quantity,
        ));
    }
    GachaDrawResponseCardResult {
        card_id: card.id.clone(),
        card_reward_results: rewards,
        duplicate_reward_type: GachaCardDuplicateRewardType::None as i32,
        ..Default::default()
    }
}

fn is_r4plus(card_id: &str) -> bool {
    Card::table().iter().any(|c| {
        c.id == card_id
            && matches!(
                CardRarity::try_from(c.rarity).unwrap_or(CardRarity::Unknown),
                CardRarity::Rarity4 | CardRarity::Rarity5
            )
    })
}

// guarantee replaces the last result with an R4+ card only when the batch rolled none.
pub fn draw(
    rng: &mut impl Rng,
    pool: &[&Card],
    count: usize,
    guarantee: bool,
    guaranteed_pool: Option<&[&Card]>,
) -> Vec<GachaDrawResponseCardResult> {
    let mut out: Vec<GachaDrawResponseCardResult> = (0..count)
        .map(|_| {
            let tier = roll_tier(rng, &RARITY_WEIGHTS);
            build_card_result(roll_card(rng, pool, tier))
        })
        .collect();
    if !guarantee {
        return out;
    }
    if !out.iter().any(|r| is_r4plus(&r.card_id)) {
        let card = match guaranteed_pool {
            Some(sel) if !sel.is_empty() => sel[rng.random_range(0..sel.len())],
            _ => {
                let r4plus = RARITY_WEIGHTS
                    .iter()
                    .filter(|(r, _)| matches!(r, CardRarity::Rarity4 | CardRarity::Rarity5))
                    .copied()
                    .collect::<Vec<_>>();
                let tier = roll_tier(rng, &r4plus);
                roll_card(rng, pool, tier)
            }
        };
        *out.last_mut().expect("count >= 1") = build_card_result(card);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use resource::master;
    use std::collections::HashSet;
    use types::enums::ResourceType;

    async fn load_pool() -> Vec<&'static Card> {
        master::load::<Card>().await.expect("card master loads");
        Card::table().iter().collect()
    }

    #[tokio::test]
    async fn per_card_uniform_hits_all_rarities() {
        let pool = load_pool().await;
        let mut rng = rand::rng();
        let results = draw(&mut rng, &pool, 50_000, false, None);
        let ids: HashSet<&str> = results.iter().map(|r| r.card_id.as_str()).collect();
        let all_ids: HashSet<&str> = pool.iter().map(|c| c.id.as_str()).collect();
        assert!(
            ids.is_subset(&all_ids),
            "draw returned ids not in Card master"
        );
        assert!(
            ids.len() > 100,
            "too few distinct cards drawn: {}",
            ids.len()
        );
        let mut rarities = HashSet::new();
        for r in &results {
            let card = pool.iter().find(|c| c.id == r.card_id).unwrap();
            rarities.insert(CardRarity::try_from(card.rarity).unwrap());
        }
        assert_eq!(
            rarities.len(),
            3,
            "expected R3/R4/R5 all to appear, got {rarities:?}"
        );
    }

    #[tokio::test]
    async fn guaranteed_slot_always_grants_r4plus() {
        let pool = load_pool().await;
        let mut rng = rand::rng();
        for _ in 0..200 {
            let results = draw(&mut rng, &pool, 10, true, None);
            assert_eq!(results.len(), 10);
            assert!(
                results.iter().any(|r| is_r4plus(&r.card_id)),
                "batch without R4+ card: {:?}",
                results.iter().map(|r| &r.card_id).collect::<Vec<_>>()
            );
        }
    }

    #[tokio::test]
    async fn no_guarantee_can_roll_r3() {
        let pool = load_pool().await;
        let mut rng = rand::rng();
        // per-card uniform => R3 appears often; 2_000 single draws must include some
        for _ in 0..2_000 {
            let results = draw(&mut rng, &pool, 1, false, None);
            assert_eq!(results.len(), 1);
            if !is_r4plus(&results[0].card_id) {
                return; // R3 (or any non-R4+) drawn: no guarantee applied
            }
        }
        panic!("2_000 single draws without guarantee all rolled R4+; guarantee is not gated");
    }

    #[tokio::test]
    async fn r5_card_result_grants_full_reward_set() {
        let pool = load_pool().await;
        let r5 = pool
            .iter()
            .copied()
            .find(|c| CardRarity::try_from(c.rarity).unwrap() == CardRarity::Rarity5)
            .expect("an R5 card exists");
        let result = build_card_result(r5);
        assert_eq!(
            result.duplicate_reward_type,
            GachaCardDuplicateRewardType::None as i32
        );
        let types: Vec<ResourceType> = result
            .card_reward_results
            .iter()
            .map(|r| ResourceType::try_from(r.resource_type).unwrap())
            .collect();
        assert!(
            types.len() >= 4,
            "R5 should grant 4+ rewards, got {types:?}"
        );
        assert_eq!(types[0], ResourceType::Card);
        assert_eq!(types[1], ResourceType::Costume);
        assert_eq!(types[2], ResourceType::SdCostume);
        assert_eq!(types[3], ResourceType::Item); // acquire_reward (sticker)
    }
}
