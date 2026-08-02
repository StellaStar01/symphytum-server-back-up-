use tonic::{Request, Response, Status};

use types::rpc::api::MasterGetResponse;
use types::rpc::api::master_server::Master;

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct MasterService {}

#[tonic::async_trait]
impl Master for MasterService {
    async fn get(&self, _req: Request<()>) -> Result<Response<MasterGetResponse>, Status> {
        Ok(Response::new(replay!(
            MasterGetResponse,
            sniffs::MASTER_GET_RESP
        )))
    }
}
