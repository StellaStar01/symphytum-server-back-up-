use resource::master::MasterTable;
use types::entity::master::Membership;
use types::enums::{ShopChargeItemSubscriptionType, ShopChargeItemType, ShopItemType, ShopType};
use types::rpc::api::common::Shop;
use types::rpc::api::common::shop::{ChargeItem, ChargeItemSubscription, Item};

/// IAP product id: hololive.dreams.com.sub.4 + the 5-digit number from the charge item id's chr- segment.
pub fn iap_id(charge_item_id: &str) -> String {
    if let Some(idx) = charge_item_id.find("chr-") {
        let rest = &charge_item_id[idx + 4..];
        if let Some(digits) = rest.split('-').next() {
            if digits.len() == 5 && digits.chars().all(|c| c.is_ascii_digit()) {
                return format!("hololive.dreams.com.sub.4{digits}");
            }
        }
    }
    charge_item_id.to_string()
}

/// membership shop: one subscription charge item per Membership master row (shared by GetShop and Shop.List).
pub fn membership_shop() -> Shop {
    Shop {
        id: "shop-subscription-membership-01".into(),
        name: "Membership".into(),
        r#type: ShopType::Membership as i32,
        items: Membership::table()
            .iter()
            .map(|m| {
                let charge_item_id = m.shop_charge_item_subscription_shop_charge_item_id.clone();
                Item {
                    r#type: ShopItemType::Charge as i32,
                    charge_item: Some(ChargeItem {
                        id: charge_item_id.clone(),
                        r#type: ShopChargeItemType::Subscription as i32,
                        subscription: Some(ChargeItemSubscription {
                            r#type: ShopChargeItemSubscriptionType::Membership as i32,
                            is_unlocked: true,
                            is_subscribed: true,
                            provide_paid_stone_quantity: 0,
                            apple_iap_product_id: iap_id(&charge_item_id),
                            google_iap_product_id: iap_id(&charge_item_id),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            })
            .collect(),
        ..Default::default()
    }
}
