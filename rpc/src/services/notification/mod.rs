use tonic::{Request, Response, Status};

use types::rpc::api::notification_server::Notification;
use types::rpc::api::{
    NotificationListResponse, NotificationReadRequest, NotificationReadResponse,
};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct NotificationService {}

#[tonic::async_trait]
impl Notification for NotificationService {
    async fn list(&self, _req: Request<()>) -> Result<Response<NotificationListResponse>, Status> {
        Ok(Response::new(replay!(
            NotificationListResponse,
            sniffs::NOTIFICATION_LIST_RESP
        )))
    }
    async fn read(
        &self,
        _req: Request<NotificationReadRequest>,
    ) -> Result<Response<NotificationReadResponse>, Status> {
        // marks the notification read; no user state to update
        Ok(Response::new(NotificationReadResponse::default()))
    }
}
