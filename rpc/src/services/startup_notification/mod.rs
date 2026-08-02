use tonic::{Request, Response, Status};

use types::rpc::api::startup_notification_server::StartupNotification;
use types::rpc::api::{
    StartupNotificationReadMusicRequest, StartupNotificationReadMusicResponse,
    StartupNotificationReadRequest, StartupNotificationReadResponse,
};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct StartupNotificationService {}

#[tonic::async_trait]
impl StartupNotification for StartupNotificationService {
    async fn read(
        &self,
        _req: Request<StartupNotificationReadRequest>,
    ) -> Result<Response<StartupNotificationReadResponse>, Status> {
        Ok(Response::new(replay!(
            StartupNotificationReadResponse,
            sniffs::STARTUP_NOTIFICATION_READ_RESP
        )))
    }

    async fn read_music(
        &self,
        _req: Request<StartupNotificationReadMusicRequest>,
    ) -> Result<Response<StartupNotificationReadMusicResponse>, Status> {
        Ok(Response::new(replay!(
            StartupNotificationReadMusicResponse,
            sniffs::STARTUP_NOTIFICATION_READ_MUSIC_RESP
        )))
    }
}
