use tonic::{Request, Response, Status};

use types::rpc::api::event_server::Event;
use types::rpc::api::{EventListEventInfoForPortalResponse, EventListEventInfoResponse};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct EventService {}

#[tonic::async_trait]
impl Event for EventService {
    async fn list_event_info(
        &self,
        _req: Request<()>,
    ) -> Result<Response<EventListEventInfoResponse>, Status> {
        Ok(Response::new(replay!(
            EventListEventInfoResponse,
            sniffs::EVENT_LIST_EVENT_INFO_RESP
        )))
    }
    async fn list_event_info_for_portal(
        &self,
        _req: Request<()>,
    ) -> Result<Response<EventListEventInfoForPortalResponse>, Status> {
        Err(Status::unimplemented("Event.list_event_info_for_portal"))
    }
}
