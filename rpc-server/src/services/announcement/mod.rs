use database::Database;
use tonic::{Request, Response, Status};
use types::Validate;
use types::rpc::api::announcement_server::Announcement;
use types::rpc::api::{
    AnnouncementListRequest, AnnouncementListResponse, AnnouncementReadRequest,
    AnnouncementReadResponse,
};

#[derive(Clone)]
pub struct AnnouncementService {
    _db: Database,
}

impl AnnouncementService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Announcement for AnnouncementService {
    async fn list(
        &self,
        req: Request<AnnouncementListRequest>,
    ) -> Result<Response<AnnouncementListResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(AnnouncementListResponse::default()))
    }

    async fn read(
        &self,
        req: Request<AnnouncementReadRequest>,
    ) -> Result<Response<AnnouncementReadResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(AnnouncementReadResponse::default()))
    }
}
