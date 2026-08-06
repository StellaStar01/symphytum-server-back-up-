use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::LoginBonusCheckResponse;
use types::rpc::api::login_bonus_server::LoginBonus;

#[derive(Clone)]
pub struct LoginBonusService {
    _db: Database,
}

impl LoginBonusService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl LoginBonus for LoginBonusService {
    async fn check(&self, _req: Request<()>) -> Result<Response<LoginBonusCheckResponse>, Status> {
        Ok(Response::new(LoginBonusCheckResponse::default()))
    }
}
