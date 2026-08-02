use tonic::{Request, Response, Status};

use types::rpc::api::LoginBonusCheckResponse;
use types::rpc::api::login_bonus_server::LoginBonus;

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct LoginBonusService {}

#[tonic::async_trait]
impl LoginBonus for LoginBonusService {
    async fn check(&self, _req: Request<()>) -> Result<Response<LoginBonusCheckResponse>, Status> {
        Ok(Response::new(replay!(
            LoginBonusCheckResponse,
            sniffs::LOGIN_BONUS_CHECK_RESP
        )))
    }
}
