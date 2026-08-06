use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::event_server::Event;
use types::rpc::api::{EventListEventInfoForPortalResponse, EventListEventInfoResponse};

#[cfg_attr(not(test), allow(dead_code))]
mod event_data;

#[derive(Clone)]
pub struct EventService {
    _db: Database,
}

impl EventService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl Event for EventService {
    async fn list_event_info(
        &self,
        _req: Request<()>,
    ) -> Result<Response<EventListEventInfoResponse>, Status> {
        // cant derive event_data.rs from master data, too bad!
        Ok(Response::new(EventListEventInfoResponse::default()))
    }
    async fn list_event_info_for_portal(
        &self,
        _req: Request<()>,
    ) -> Result<Response<EventListEventInfoForPortalResponse>, Status> {
        Ok(Response::new(EventListEventInfoForPortalResponse::default()))
    }
}
