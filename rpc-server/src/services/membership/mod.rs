use database::Database;
use resource::master;
use tonic::{Request, Response, Status};
use types::entity::master::Membership as MembershipMaster;
use types::rpc::api::MembershipGetShopResponse;
use types::rpc::api::membership_server::Membership;

pub mod membership_data;

use membership_data::membership_shop;

#[derive(Clone)]
pub struct MembershipService {
    _db: Database,
}

impl MembershipService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<MembershipMaster>().await?;
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Membership for MembershipService {
    async fn get_shop(
        &self,
        _req: Request<()>,
    ) -> Result<Response<MembershipGetShopResponse>, Status> {
        Ok(Response::new(MembershipGetShopResponse {
            shop: Some(membership_shop()),
            common_response: None,
        }))
    }
}
