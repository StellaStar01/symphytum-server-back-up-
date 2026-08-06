use database::Database;
use tonic::{Request, Response, Status};

use types::Validate;
use types::rpc::api::auth_server::Auth;
use types::rpc::api::{AuthCreateResponse, AuthLoginRequest, AuthLoginResponse};

use crate::auth_token;

#[derive(Clone)]
pub struct AuthService {
    db: Database,
}

impl AuthService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn login(
        &self,
        req: Request<AuthLoginRequest>,
    ) -> Result<Response<AuthLoginResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let uid = self
            .db
            .get_or_create_account(&req.credential)
            .await
            .map_err(|e| Status::internal(format!("account: {e}")))?;

        let token = auth_token::mint(&uid);
        tracing::info!("issued token for {uid}: {token}");

        Ok(Response::new(AuthLoginResponse {
            game_auth_token: token,
            is_play_integrity_check_required: false,
            play_integrity_nonce: String::new(),
        }))
    }

    async fn create(&self, _req: Request<()>) -> Result<Response<AuthCreateResponse>, Status> {
        Err(Status::unimplemented("Auth.create"))
    }
}
