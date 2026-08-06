use prost::Message;
use tonic::{Request, Response, Status};

use database::Database;
use resource::master::MASTER_GET_BIN;
use types::rpc::api::MasterGetResponse;
use types::rpc::api::master_server::Master;

#[derive(Clone)]
pub struct MasterService {
    _db: Database,
}

impl MasterService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Master for MasterService {
    async fn get(&self, _req: Request<()>) -> Result<Response<MasterGetResponse>, Status> {
        if MASTER_GET_BIN.is_empty() {
            tracing::error!("master_get.bin is empty; run the ./bin master to dump it");
            return Ok(Response::new(MasterGetResponse::default()));
        }
        let mut resp = MasterGetResponse::decode(MASTER_GET_BIN)
            .map_err(|e| Status::internal(format!("master_get.bin decode: {e}")))?;
        resp.version = "9f9f9f9f61dd6900cac86860443e3dfea22b8efe5ccc424b3b99a67821acc3be".into();
        Ok(Response::new(resp))
    }
}
