use tonic::{Request, Response, Status};

use types::rpc::api::user_server::User;
use types::rpc::api::{UserDeleteResponse, UserGetResponse};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct UserService {}

#[tonic::async_trait]
impl User for UserService {
    async fn get(&self, _req: Request<()>) -> Result<Response<UserGetResponse>, Status> {
        Ok(Response::new(replay!(
            UserGetResponse,
            sniffs::USER_GET_RESP
        )))
    }
    async fn delete(&self, _req: Request<()>) -> Result<Response<UserDeleteResponse>, Status> {
        Err(Status::unimplemented("User.delete"))
    }
}
