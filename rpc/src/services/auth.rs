use tonic::{Request, Response, Status};

use types::rpc::api::auth_server::Auth;
use types::rpc::api::{AuthCreateResponse, AuthLoginRequest, AuthLoginResponse};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct AuthService {}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn login(
        &self,
        _req: Request<AuthLoginRequest>,
    ) -> Result<Response<AuthLoginResponse>, Status> {
        Ok(Response::new(replay!(
            AuthLoginResponse,
            sniffs::AUTH_LOGIN_RESP
        )))
    }
    async fn create(&self, _req: Request<()>) -> Result<Response<AuthCreateResponse>, Status> {
        Err(Status::unimplemented("Auth.create"))
    }
}
