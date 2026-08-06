use tonic::transport::Server;

use database::Database;

use types::rpc::api::announcement_server::AnnouncementServer;
use types::rpc::api::auth_server::AuthServer;
use types::rpc::api::card_server::CardServer;
use types::rpc::api::character_history_server::CharacterHistoryServer;
use types::rpc::api::character_server::CharacterServer;
use types::rpc::api::costume_server::CostumeServer;
use types::rpc::api::event_server::EventServer;
use types::rpc::api::exchange_server::ExchangeServer;
use types::rpc::api::gacha_server::GachaServer;
use types::rpc::api::gift_server::GiftServer;
use types::rpc::api::home_server::HomeServer;
use types::rpc::api::jump_rope_server::JumpRopeServer;
use types::rpc::api::live_server::LiveServer;
use types::rpc::api::login_bonus_server::LoginBonusServer;
use types::rpc::api::master_server::MasterServer;
use types::rpc::api::membership_server::MembershipServer;
use types::rpc::api::multi_game_server::MultiGameServer;
use types::rpc::api::notice_server::NoticeServer;
use types::rpc::api::notification_server::NotificationServer;
use types::rpc::api::park_server::ParkServer;
use types::rpc::api::profile_server::ProfileServer;
use types::rpc::api::shop_server::ShopServer;
use types::rpc::api::skill_tree_server::SkillTreeServer;
use types::rpc::api::startup_notification_server::StartupNotificationServer;
use types::rpc::api::system_server::SystemServer;
use types::rpc::api::tutorial_server::TutorialServer;
use types::rpc::api::user_content_cdn_server::UserContentCdnServer;
use types::rpc::api::user_server::UserServer;

use resource::cert;
use resource::config::{self, CONFIG, repo_root};

mod auth_token;
mod quali_crypt;
mod services;

use services::announcement::AnnouncementService;
use services::auth::AuthService;
use services::card::CardService;
use services::character::CharacterService;
use services::character_history::CharacterHistoryService;
use services::costume::CostumeService;
use services::event::EventService;
use services::exchange::ExchangeService;
use services::gacha::GachaService;
use services::gift::GiftService;
use services::home::HomeService;
use services::jump_rope::JumpRopeService;
use services::live::LiveService;
use services::login_bonus::LoginBonusService;
use services::master::MasterService;
use services::membership::MembershipService;
use services::multi_game::MultiGameService;
use services::notice::NoticeService;
use services::notification::NotificationService;
use services::park::ParkService;
use services::profile::ProfileService;
use services::shop::ShopService;
use services::skill_tree::SkillTreeService;
use services::startup_notification::StartupNotificationService;
use services::system::SystemService;
use services::tutorial::TutorialService;
use services::user::UserService;
use services::user_content_cdn::UserContentCdnService;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::init_tracing();

    let db = Database::open(&repo_root().join("symphytum.db")).await?;
    tracing::info!("database ready at {}/symphytum.db", repo_root().display());

    let addr = format!("{}:{}", CONFIG.rpc_server.host, CONFIG.rpc_server.port).parse()?;
    let layer = config::debug_layer();
    let server = Server::builder().tls_config(cert::get_tls_config().await?)?;
    tracing::info!("serving symphytum on {}", addr);

    let server = server.layer(quali_crypt::QualiCryptLayer);

    let mut server = server.layer(layer);

    server
        .add_service(SystemServer::new(SystemService::init(db.clone()).await?))
        .add_service(MasterServer::new(MasterService::init(db.clone()).await?))
        .add_service(AuthServer::new(AuthService::init(db.clone()).await?))
        .add_service(UserServer::new(UserService::init(db.clone()).await?))
        .add_service(HomeServer::new(HomeService::init(db.clone()).await?))
        .add_service(LoginBonusServer::new(
            LoginBonusService::init(db.clone()).await?,
        ))
        .add_service(MultiGameServer::new(
            MultiGameService::init(db.clone()).await?,
        ))
        .add_service(ParkServer::new(ParkService::init(db.clone()).await?))
        .add_service(EventServer::new(EventService::init(db.clone()).await?))
        .add_service(ExchangeServer::new(
            ExchangeService::init(db.clone()).await?,
        ))
        .add_service(NotificationServer::new(
            NotificationService::init(db.clone()).await?,
        ))
        .add_service(StartupNotificationServer::new(
            StartupNotificationService::init(db.clone()).await?,
        ))
        .add_service(CardServer::new(CardService::init(db.clone()).await?))
        .add_service(CharacterServer::new(
            CharacterService::init(db.clone()).await?,
        ))
        .add_service(CharacterHistoryServer::new(
            CharacterHistoryService::init(db.clone()).await?,
        ))
        .add_service(CostumeServer::new(CostumeService::init(db.clone()).await?))
        .add_service(GachaServer::new(GachaService::init(db.clone()).await?))
        .add_service(LiveServer::new(LiveService::init(db.clone()).await?))
        .add_service(TutorialServer::new(
            TutorialService::init(db.clone()).await?,
        ))
        .add_service(AnnouncementServer::new(
            AnnouncementService::init(db.clone()).await?,
        ))
        .add_service(GiftServer::new(GiftService::init(db.clone()).await?))
        .add_service(JumpRopeServer::new(
            JumpRopeService::init(db.clone()).await?,
        ))
        .add_service(MembershipServer::new(
            MembershipService::init(db.clone()).await?,
        ))
        .add_service(NoticeServer::new(NoticeService::init(db.clone()).await?))
        .add_service(ProfileServer::new(ProfileService::init(db.clone()).await?))
        .add_service(ShopServer::new(ShopService::init(db.clone()).await?))
        .add_service(SkillTreeServer::new(
            SkillTreeService::init(db.clone()).await?,
        ))
        .add_service(UserContentCdnServer::new(
            UserContentCdnService::init(db.clone()).await?,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
