use types::entity::transaction::{UserBalanceFree, UserBalancePaid, UserItem, UserItemOwned};
use types::rpc::api::common::UserData;

// hardcoded wallet
// injected into the User/Get response (`user_data`),
pub const FREE_DIAMONDS: i32 = 999_999; // blue diamonds (resource 1001)
pub const PAID_DIAMONDS: i32 = 999_999; // red diamonds (resource 1002)
pub const GACHA_TICKET_QUANTITY: i64 = 999;

const GACHA_TICKET_IDS: [&str; 3] = [
    "item-gacha_ticket-cmn-01", // common-normal gacha
    "item-gacha_ticket-r4-01",  // gacha-common-ticket_r4
    "item-gacha_ticket-r5-01",  // gacha-common-ticket_r5
];

pub fn patch(data: &mut UserData) {
    data.user_balance_free = Some(UserBalanceFree {
        free_quantity: FREE_DIAMONDS,
    });
    data.user_balance_paid = Some(UserBalancePaid {
        paid_quantity: PAID_DIAMONDS,
    });
    for id in GACHA_TICKET_IDS {
        match data.user_item_list.iter_mut().find(|i| i.item_id == id) {
            Some(item) => item.quantity = GACHA_TICKET_QUANTITY,
            None => data.user_item_list.push(UserItem {
                item_id: id.to_string(),
                expired_time: 0,
                quantity: GACHA_TICKET_QUANTITY,
                last_acquired_time: 0,
            }),
        }
        if !data.user_item_owned_list.iter().any(|i| i.item_id == id) {
            data.user_item_owned_list.push(UserItemOwned {
                item_id: id.to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use types::rpc::api::UserGetResponse;

    use crate::sniffs;

    #[test]
    fn patch_injects_wallet_into_user_get() {
        let mut resp = UserGetResponse::decode(sniffs::USER_GET_RESP).expect("sniff decodes");
        let data = resp.user_data.as_mut().expect("user_data present");
        patch(data);
        assert_eq!(
            data.user_balance_free.as_ref().unwrap().free_quantity,
            FREE_DIAMONDS
        );
        assert_eq!(
            data.user_balance_paid.as_ref().unwrap().paid_quantity,
            PAID_DIAMONDS
        );
        for id in GACHA_TICKET_IDS {
            let item = data
                .user_item_list
                .iter()
                .find(|i| i.item_id == id)
                .unwrap_or_else(|| panic!("ticket {id} missing from user_item_list"));
            assert_eq!(item.quantity, GACHA_TICKET_QUANTITY);
            assert!(data.user_item_owned_list.iter().any(|i| i.item_id == id));
        }
        // still encodes as valid protobuf
        assert!(resp.encode_to_vec().len() > 0);
    }
}
