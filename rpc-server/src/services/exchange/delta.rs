use types::rpc::api::common::UserData;

/// post-booth-read/purchase delta: the booth's rows in user_exchange_booth_list.
pub fn after_exchange_booth_read(updated: &UserData, booth_id: &str) -> UserData {
    let mut d = UserData::default();
    d.user_exchange_booth_list = updated
        .user_exchange_booth_list
        .iter()
        .filter(|b| b.exchange_booth_id == booth_id)
        .cloned()
        .collect();
    d
}
