use database::Database;
use resource::master;
use tonic::{Request, Response, Status};
use types::Validate;
use types::entity::master::ShopChargeItemProduct;
use types::rpc::api::common::{Response as CommonResponse, UserData};
use types::rpc::api::shop_server::Shop;
use types::rpc::api::{
    ShopListResponse, ShopPurchaseConsumptionItemRequest, ShopPurchaseConsumptionItemResponse,
    ShopReadRequest, ShopReadResponse,
};

mod shop_data;

use shop_data::shops;

#[derive(Clone)]
pub struct ShopService {
    _db: Database,
}

impl ShopService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<ShopChargeItemProduct>().await?;
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Shop for ShopService {
    async fn list(&self, _req: Request<()>) -> Result<Response<ShopListResponse>, Status> {
        Ok(Response::new(ShopListResponse {
            shops: shops(),
            common_response: None,
        }))
    }

    async fn read(
        &self,
        req: Request<ShopReadRequest>,
    ) -> Result<Response<ShopReadResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if !shops().iter().any(|s| s.id == req.shop_id) {
            return Err(Status::invalid_argument("unknown shop_id"));
        }
        // the client already holds payment orders from user_data; it only needs a success response
        Ok(Response::new(ShopReadResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(UserData::default()),
                ..Default::default()
            }),
        }))
    }

    async fn purchase_consumption_item(
        &self,
        req: Request<ShopPurchaseConsumptionItemRequest>,
    ) -> Result<Response<ShopPurchaseConsumptionItemResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(ShopPurchaseConsumptionItemResponse::default()))
    }
}
