use tonic::{Request, Response, Status};

use types::rpc::api::system_server::System;
use types::rpc::api::{SystemGetSystemInfoRequest, SystemGetSystemInfoResponse};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct SystemService {}

#[tonic::async_trait]
impl System for SystemService {
    async fn get_system_info(
        &self,
        _req: Request<SystemGetSystemInfoRequest>,
    ) -> Result<Response<SystemGetSystemInfoResponse>, Status> {
        Ok(Response::new(replay!(
            SystemGetSystemInfoResponse,
            sniffs::SYSTEM_GET_SYSTEM_INFO_RESP
        )))
    }
}
