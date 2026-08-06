use database::Database;
use resource::master;
use tonic::{Request, Response, Status};
use types::Validate;
use types::entity::master::{
    Card, ExchangeBooth, ExchangeBoothFixedItem, ExchangeBoothGroup, GachaPoint, Membership,
};
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::exchange_server::Exchange;
use types::rpc::api::{
    ExchangeListRequest, ExchangeListResponse, ExchangePurchaseRequest, ExchangePurchaseResponse,
    ExchangeReadBoothRequest, ExchangeReadBoothResponse,
};

mod delta;
pub mod exchange_data;
mod purchase;

use crate::auth_token;

#[derive(Clone)]
pub struct ExchangeService {
    db: Database,
}

impl ExchangeService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<Card>().await?;
        master::load::<ExchangeBooth>().await?;
        master::load::<ExchangeBoothFixedItem>().await?;
        master::load::<ExchangeBoothGroup>().await?;
        master::load::<GachaPoint>().await?;
        master::load::<Membership>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Exchange for ExchangeService {
    async fn list(
        &self,
        req: Request<ExchangeListRequest>,
    ) -> Result<Response<ExchangeListResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(exchange_data::group_response(
            &req.booth_group_id,
        )))
    }

    async fn purchase(
        &self,
        req: Request<ExchangePurchaseRequest>,
    ) -> Result<Response<ExchangePurchaseResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(
            purchase::purchase(&self.db, &uid, req).await?,
        ))
    }

    async fn read_booth(
        &self,
        req: Request<ExchangeReadBoothRequest>,
    ) -> Result<Response<ExchangeReadBoothResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let booth_id = &req.exchange_booth_id;
        let now = database::unix_now();
        self.db
            .exchange_booth_read(&uid, booth_id, now)
            .await
            .map_err(|e| Status::internal(format!("booth read: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(ExchangeReadBoothResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_exchange_booth_read(&updated, booth_id)),
                ..Default::default()
            }),
        }))
    }
}
