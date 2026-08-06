use types::rpc::api::common::UserData;

pub fn after_profile(updated: &UserData) -> UserData {
    UserData {
        user_profile: updated.user_profile.clone(),
        ..Default::default()
    }
}
