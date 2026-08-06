use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::startup_notification_server::StartupNotification;
use types::rpc::api::{
    StartupNotificationReadMusicRequest, StartupNotificationReadMusicResponse,
    StartupNotificationReadRequest, StartupNotificationReadResponse,
};

#[derive(Clone)]
pub struct StartupNotificationService {
    _db: Database,
}

impl StartupNotificationService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl StartupNotification for StartupNotificationService {
    async fn read(
        &self,
        _req: Request<StartupNotificationReadRequest>,
    ) -> Result<Response<StartupNotificationReadResponse>, Status> {
        Ok(Response::new(StartupNotificationReadResponse::default()))
    }

    async fn read_music(
        &self,
        _req: Request<StartupNotificationReadMusicRequest>,
    ) -> Result<Response<StartupNotificationReadMusicResponse>, Status> {
        Ok(Response::new(
            StartupNotificationReadMusicResponse::default(),
        ))
    }
}
