use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::user_server::User;
use types::rpc::api::{UserDeleteResponse, UserGetResponse};

use crate::auth_token;

#[derive(Clone)]
pub struct UserService {
    db: Database,
}

impl UserService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl User for UserService {
    async fn get(&self, req: Request<()>) -> Result<Response<UserGetResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let data = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("load user: {e}")))?;
        Ok(Response::new(UserGetResponse {
            user_data: Some((*data).clone()),
        }))
    }
    async fn delete(&self, _req: Request<()>) -> Result<Response<UserDeleteResponse>, Status> {
        Err(Status::unimplemented("User.delete"))
    }
}
