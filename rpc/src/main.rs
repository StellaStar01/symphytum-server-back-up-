use tonic::transport::Server;

use types::rpc::api::auth_server::AuthServer;
use types::rpc::api::event_server::EventServer;
use types::rpc::api::gacha_server::GachaServer;
use types::rpc::api::home_server::HomeServer;
use types::rpc::api::login_bonus_server::LoginBonusServer;
use types::rpc::api::master_server::MasterServer;
use types::rpc::api::multi_game_server::MultiGameServer;
use types::rpc::api::notification_server::NotificationServer;
use types::rpc::api::park_server::ParkServer;
use types::rpc::api::startup_notification_server::StartupNotificationServer;
use types::rpc::api::system_server::SystemServer;
use types::rpc::api::user_server::UserServer;

use resource::cert;
use resource::config::{self, CONFIG};

mod services;
mod sniffs;

#[cfg(feature = "quali-crypt")]
mod quali_crypt;

use services::auth::AuthService;
use services::event::EventService;
use services::gacha::GachaService;
use services::home::HomeService;
use services::login_bonus::LoginBonusService;
use services::master::MasterService;
use services::multi_game::MultiGameService;
use services::notification::NotificationService;
use services::park::ParkService;
use services::startup_notification::StartupNotificationService;
use services::system::SystemService;
use services::user::UserService;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::init_tracing();

    // should just load the ones we need because
    // without these its only 0.8MB ram usage (release)
    // but with them its 125.7MB
    // tracing::info!("loading master jsons...");
    // let master_n = resource::master::load_all().await?;
    // tracing::info!("loaded {} master json", master_n);

    let addr = format!("{}:{}", CONFIG.rpc.host, CONFIG.rpc.port).parse()?;
    let layer = config::debug_layer();
    let server = Server::builder().tls_config(cert::get_tls_config().await?)?;
    tracing::info!("serving symphytum on {}", addr);

    #[cfg(feature = "quali-crypt")]
    let server = server.layer(quali_crypt::QualiCryptLayer);

    let mut server = server.layer(layer);

    server
        .add_service(SystemServer::new(SystemService::default()))
        .add_service(MasterServer::new(MasterService::default()))
        .add_service(AuthServer::new(AuthService::default()))
        .add_service(UserServer::new(UserService::default()))
        .add_service(HomeServer::new(HomeService::default()))
        .add_service(LoginBonusServer::new(LoginBonusService::default()))
        .add_service(MultiGameServer::new(MultiGameService::default()))
        .add_service(ParkServer::new(ParkService::default()))
        .add_service(EventServer::new(EventService::default()))
        .add_service(NotificationServer::new(NotificationService::default()))
        .add_service(StartupNotificationServer::new(
            StartupNotificationService::default(),
        ))
        .add_service(GachaServer::new(GachaService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
