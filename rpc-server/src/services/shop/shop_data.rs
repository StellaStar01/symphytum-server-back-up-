use types::enums::{ResetIntervalType, ShopChargeItemType, ShopItemType, ShopType};
use types::rpc::api::common::Shop;
use types::rpc::api::common::shop::{ChargeItem, ChargeItemConsumable, Item};

use crate::services::membership::membership_data::membership_shop;

/// fake stone shop; cant derive from master data, nonexistent
fn stone_shop() -> Shop {
    let item = |id: &str, stone: i32, asset: &str, iap: &str, price: &str| Item {
        r#type: ShopItemType::Charge as i32,
        charge_item: Some(ChargeItem {
            id: id.into(),
            r#type: ShopChargeItemType::Consumable as i32,
            consumable: Some(ChargeItemConsumable {
                name: "Release Celebration!\nDiamond Sale".into(),
                provide_paid_stone_quantity: stone,
                rewards: vec![],
                is_unlocked: true,
                unlock_condition_group_id: String::new(),
                end_time: 1_788_724_799_000,
                reset_interval_type: ResetIntervalType::None as i32,
                next_reset_time: 0,
                limit_count: 1,
                purchased_count: 0,
                asset_id: asset.into(),
                last_reset_time: 0,
                is_background_special: true,
                color: String::new(),
                is_new: false,
                apple_iap_product_id: iap.into(),
                google_iap_product_id: iap.into(),
                steam_mtx_product_id: iap.trim_start_matches("hololive.dreams.com.con.").into(),
                steam_price: price.into(),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    Shop {
        id: "shop-stone-01".into(),
        r#type: ShopType::Stone as i32,
        name: "Diamond".into(),
        items: vec![
            item(
                "shop_charge_item_consumable-stone-01-sp-release-01",
                720,
                "stone-special-release-001",
                "hololive.dreams.com.con.200002",
                "USD 1.00",
            ),
            item(
                "shop_charge_item_consumable-stone-01-sp-release-02",
                3500,
                "stone-special-release-002",
                "hololive.dreams.com.con.200003",
                "USD 1.00",
            ),
            item(
                "shop_charge_item_consumable-stone-01-sp-release-03",
                12300,
                "stone-special-release-003",
                "hololive.dreams.com.con.200004",
                "USD 1.00",
            ),
        ],
        ..Default::default()
    }
}

pub fn shops() -> Vec<Shop> {
    vec![stone_shop(), membership_shop()]
}
