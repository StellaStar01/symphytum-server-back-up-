use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::notification_server::Notification;
use types::rpc::api::{
    NotificationListResponse, NotificationReadRequest, NotificationReadResponse,
};

use crate::auth_token;

#[derive(Clone)]
pub struct NotificationService {
    _db: Database,
}

impl NotificationService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Notification for NotificationService {
    async fn list(&self, req: Request<()>) -> Result<Response<NotificationListResponse>, Status> {
        let mut resp = NotificationListResponse::default();
        // these are false by default but i like explicit
        resp.is_gacha_unread = false;
        resp.is_notice_unread = false;
        resp.is_friend_offer_received = false;
        resp.is_friend_exists = false;
        resp.is_membership_unread = false;
        resp.is_shop_item_unread = false;
        if auth_token::uid_opt(&req).is_some() {
            resp.common_response = Some(CommonResponse {
                updated_data: Some(types::rpc::api::common::UserData::default()),
                ..Default::default()
            });
        }
        Ok(Response::new(resp))
    }
    async fn read(
        &self,
        _req: Request<NotificationReadRequest>,
    ) -> Result<Response<NotificationReadResponse>, Status> {
        // marks the notification read; no user state to update
        Ok(Response::new(NotificationReadResponse::default()))
    }
}
