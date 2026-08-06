use database::Database;
use tonic::{Request, Response, Status};

use types::Validate;
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::common::UserContentCdnSignedCookie;
use types::rpc::api::home_login_response::{
    ExpiredResourceResult, RealtimeNotificationConnectionInfo,
};
use types::rpc::api::home_server::Home;
use types::rpc::api::{HomeAgreeRuleRequest, HomeAgreeRuleResponse, HomeLoginResponse};

use crate::auth_token;

#[derive(Clone)]
pub struct HomeService {
    db: Database,
}

impl HomeService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Home for HomeService {
    async fn login(&self, req: Request<()>) -> Result<Response<HomeLoginResponse>, Status> {
        let mut resp = HomeLoginResponse {
            expired_resource_result: Some(ExpiredResourceResult::default()),
            fcm_topics: vec!["ALL".into(), "REGION_AS".into(), "LOGIN_2026_08_01".into()],
            user_content_cdn_signed_cookie: Some(UserContentCdnSignedCookie {
                policy: "URLPrefix=aHR0cHM6Ly91c2VyLWNvbnRlbnQuZ2FtZS1ob2xvbGl2ZS1kcmVhbXMuY29tLw==:Expires=1785653715:KeyName=dreams-user-content-cdn-signed-url-key".into(),
                signature: "YjtHzkK2wC1ulBx-8nlVtzn88vo".into(),
                key_name: "dreams-user-content-cdn-signed-url-key".into(),
                expired_time_milliseconds: 1785653715490,
            }),
            realtime_notification_connection_info: Some(RealtimeNotificationConnectionInfo {
                sse_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.symphytum-sse-dummy-token".into(),
                sse_url: "https://realtime-notification.game-hololive-dreams.com/user/stream".into(),
            }),
            ..Default::default()
        };
        // user state is the live account snapshot
        if let Some(uid) = auth_token::uid_opt(&req) {
            let data = self
                .db
                .user_data(&uid)
                .await
                .map_err(|e| Status::internal(format!("load user: {e}")))?;

            let mut updated = types::rpc::api::common::UserData::default();
            updated.user_profile = data.user_profile.clone();
            updated.user_time = data.user_time.clone();
            resp.common_response = Some(CommonResponse {
                updated_data: Some(updated),
                ..Default::default()
            });
        }
        Ok(Response::new(resp))
    }
    async fn agree_rule(
        &self,
        req: Request<HomeAgreeRuleRequest>,
    ) -> Result<Response<HomeAgreeRuleResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(HomeAgreeRuleResponse::default()))
    }
}
