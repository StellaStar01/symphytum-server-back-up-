use tonic::{Request, Response, Status};

use types::rpc::api::home_server::Home;
use types::rpc::api::{HomeAgreeRuleRequest, HomeAgreeRuleResponse, HomeLoginResponse};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct HomeService {}

#[tonic::async_trait]
impl Home for HomeService {
    async fn login(&self, _req: Request<()>) -> Result<Response<HomeLoginResponse>, Status> {
        Ok(Response::new(replay!(
            HomeLoginResponse,
            sniffs::HOME_LOGIN_RESP
        )))
    }
    async fn agree_rule(
        &self,
        _req: Request<HomeAgreeRuleRequest>,
    ) -> Result<Response<HomeAgreeRuleResponse>, Status> {
        Err(Status::unimplemented("Home.agree_rule"))
    }
}
