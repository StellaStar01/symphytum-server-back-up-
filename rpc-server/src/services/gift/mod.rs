use database::Database;
use tonic::{Request, Response, Status};
use types::Validate;
use types::rpc::api::gift_server::Gift;
use types::rpc::api::{
    GiftCountResponse, GiftListHistoryResponse, GiftListRequest, GiftListResponse, GiftOpenRequest,
    GiftOpenResponse, GiftTopResponse,
};

#[derive(Clone)]
pub struct GiftService {
    _db: Database,
}

impl GiftService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Gift for GiftService {
    async fn top(&self, _req: Request<()>) -> Result<Response<GiftTopResponse>, Status> {
        Ok(Response::new(GiftTopResponse::default()))
    }

    async fn list(
        &self,
        req: Request<GiftListRequest>,
    ) -> Result<Response<GiftListResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(GiftListResponse::default()))
    }

    async fn open(
        &self,
        req: Request<GiftOpenRequest>,
    ) -> Result<Response<GiftOpenResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(GiftOpenResponse {
            unopened_gift_ids: req.gift_ids,
            ..Default::default()
        }))
    }

    async fn list_history(
        &self,
        _req: Request<()>,
    ) -> Result<Response<GiftListHistoryResponse>, Status> {
        Ok(Response::new(GiftListHistoryResponse::default()))
    }

    async fn count(&self, _req: Request<()>) -> Result<Response<GiftCountResponse>, Status> {
        Ok(Response::new(GiftCountResponse::default()))
    }
}
