/// pickup rate-up cards: the first 5 R5 cards by master order, so promotions match the exchange.
pub fn pickup_cards() -> Vec<String> {
    crate::services::exchange::exchange_data::r5_cards_ordered()
        .into_iter()
        .take(5)
        .collect()
}
