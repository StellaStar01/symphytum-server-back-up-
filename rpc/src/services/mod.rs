pub mod auth;
pub mod event;
pub mod gacha;
pub mod home;
pub mod login_bonus;
pub mod master;
pub mod multi_game;
pub mod notification;
pub mod park;
pub mod startup_notification;
pub mod system;
pub mod user;

macro_rules! replay {
    ($resp:ty, $bin:expr) => {{
        static RESP: ::std::sync::LazyLock<$resp> = ::std::sync::LazyLock::new(|| {
            <$resp as ::prost::Message>::decode($bin).expect("hardcoded sniff bin must decode")
        });
        RESP.clone()
    }};
}
pub(crate) use replay;
