use types::entity::master::Card;
use types::enums::CardRarity;
use types::rpc::api::{GachaCardProbability, GachaRarityProbability};

/// Drop rates. edit these to rebalance. Weights are parts per ten million per
/// rarity tier; cards within a tier are equally likely. Current values
/// (54/54/59 = Card.json tier counts) make every card equally likely.
/// The probability builders normalize the table to a 100M total at runtime.
pub const RARITY_WEIGHTS: [(CardRarity, u64); 3] = [
    (CardRarity::Rarity3, 54),
    (CardRarity::Rarity4, 54),
    (CardRarity::Rarity5, 59),
];

// Normalized parts-per-ten-million for one tier. The LAST listed tier absorbs
// rounding so all listed tiers sum to exactly 100_000_000.
pub fn tier_parts(rarity: CardRarity, tiers: &[(CardRarity, u64)]) -> u64 {
    let total: u64 = tiers.iter().map(|(_, w)| w).sum();
    let idx = tiers
        .iter()
        .position(|(r, _)| *r == rarity)
        .expect("tier in table");
    if idx + 1 == tiers.len() {
        100_000_000
            - tiers[..idx]
                .iter()
                .map(|(r, _)| tier_parts(*r, tiers))
                .sum::<u64>()
    } else {
        tiers[idx].1 * 100_000_000 / total
    }
}

fn card_probabilities(pool: &[&Card], rarity: CardRarity) -> Vec<GachaCardProbability> {
    let tier_cards = cards_of_rarity(pool, rarity);
    let per_card = tier_parts(rarity, &RARITY_WEIGHTS) / tier_cards.len().max(1) as u64;
    tier_cards
        .into_iter()
        .map(|c| GachaCardProbability {
            card_id: c.id.clone(),
            parts_per_ten_million_probability: per_card as i32,
            is_rate_up: false,
        })
        .collect()
}

pub fn cards_of_rarity<'a>(pool: &'a [&'a Card], rarity: CardRarity) -> Vec<&'a Card> {
    pool.iter()
        .copied()
        .filter(|c| CardRarity::try_from(c.rarity).ok() == Some(rarity))
        .collect()
}

// normal tiers = all three; fixed tiers = R4/R5 (the guaranteed slot's pool).
pub fn probability_response(
    pool: &[&Card],
) -> (Vec<GachaRarityProbability>, Vec<GachaRarityProbability>) {
    let normal = RARITY_WEIGHTS
        .iter()
        .map(|(r, _)| GachaRarityProbability {
            rarity: *r as i32,
            parts_per_ten_million_probability: tier_parts(*r, &RARITY_WEIGHTS) as i32,
            card_probabilities: card_probabilities(pool, *r),
        })
        .collect();
    let fixed_tiers = RARITY_WEIGHTS
        .iter()
        .filter(|(r, _)| matches!(r, CardRarity::Rarity4 | CardRarity::Rarity5))
        .copied()
        .collect::<Vec<_>>();
    let fixed = fixed_tiers
        .iter()
        .map(|(r, _)| GachaRarityProbability {
            rarity: *r as i32,
            parts_per_ten_million_probability: tier_parts(*r, &fixed_tiers) as i32,
            card_probabilities: card_probabilities(pool, *r),
        })
        .collect();
    (normal, fixed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use resource::master;
    use resource::master::MasterTable;

    #[tokio::test]
    async fn rarity_weights_normalize_to_100m() {
        master::load::<Card>().await.expect("card master loads");
        let sum: u64 = RARITY_WEIGHTS
            .iter()
            .map(|(r, _)| tier_parts(*r, &RARITY_WEIGHTS))
            .sum();
        assert_eq!(sum, 100_000_000);

        let pool: Vec<&Card> = Card::table().iter().collect();
        let (normal, _) = probability_response(&pool);
        let normal_sum: i64 = normal
            .iter()
            .map(|t| t.parts_per_ten_million_probability as i64)
            .sum();
        assert_eq!(normal_sum, 100_000_000);
        assert_eq!(normal.len(), 3);
    }
}
