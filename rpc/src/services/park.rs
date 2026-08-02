use tonic::{Request, Response, Status};

use types::rpc::api::park_server::Park;
use types::rpc::api::{
    ParkCollectFixedSymbolRequest, ParkCollectFixedSymbolResponse, ParkCollectRandomSymbolRequest,
    ParkCollectRandomSymbolResponse, ParkEnterResponse, ParkListenCallRequest,
    ParkListenCallResponse, ParkOpenTreasureRequest, ParkOpenTreasureResponse,
    ParkReadTalkFreeRequest, ParkReadTalkFreeResponse, ParkReceivePlayerLevelRewardRequest,
    ParkReceivePlayerLevelRewardResponse, ParkRefreshRequest, ParkRefreshResponse,
    ParkReportActionRequest, ParkReportActionResponse, ParkSelectAreaRequest,
    ParkSelectAreaResponse, ParkSetAccessoryRequest, ParkSetAccessoryResponse,
    ParkSetCharacterRequest, ParkSetCharacterResponse, ParkUpdateTimePeriodRequest,
    ParkUpdateTimePeriodResponse,
};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct ParkService {}

#[tonic::async_trait]
impl Park for ParkService {
    async fn enter(&self, _req: Request<()>) -> Result<Response<ParkEnterResponse>, Status> {
        Ok(Response::new(replay!(
            ParkEnterResponse,
            sniffs::PARK_ENTER_RESP
        )))
    }

    async fn refresh(
        &self,
        _req: Request<ParkRefreshRequest>,
    ) -> Result<Response<ParkRefreshResponse>, Status> {
        Ok(Response::new(replay!(
            ParkRefreshResponse,
            sniffs::PARK_REFRESH_RESP
        )))
    }
    async fn set_character(
        &self,
        _req: Request<ParkSetCharacterRequest>,
    ) -> Result<Response<ParkSetCharacterResponse>, Status> {
        Err(Status::unimplemented("Park.set_character"))
    }

    async fn set_accessory(
        &self,
        _req: Request<ParkSetAccessoryRequest>,
    ) -> Result<Response<ParkSetAccessoryResponse>, Status> {
        Err(Status::unimplemented("Park.set_accessory"))
    }

    async fn update_time_period(
        &self,
        _req: Request<ParkUpdateTimePeriodRequest>,
    ) -> Result<Response<ParkUpdateTimePeriodResponse>, Status> {
        Err(Status::unimplemented("Park.update_time_period"))
    }

    async fn receive_player_level_reward(
        &self,
        _req: Request<ParkReceivePlayerLevelRewardRequest>,
    ) -> Result<Response<ParkReceivePlayerLevelRewardResponse>, Status> {
        Err(Status::unimplemented("Park.receive_player_level_reward"))
    }

    async fn open_treasure(
        &self,
        _req: Request<ParkOpenTreasureRequest>,
    ) -> Result<Response<ParkOpenTreasureResponse>, Status> {
        Err(Status::unimplemented("Park.open_treasure"))
    }

    async fn collect_fixed_symbol(
        &self,
        _req: Request<ParkCollectFixedSymbolRequest>,
    ) -> Result<Response<ParkCollectFixedSymbolResponse>, Status> {
        Err(Status::unimplemented("Park.collect_fixed_symbol"))
    }

    async fn collect_random_symbol(
        &self,
        _req: Request<ParkCollectRandomSymbolRequest>,
    ) -> Result<Response<ParkCollectRandomSymbolResponse>, Status> {
        Err(Status::unimplemented("Park.collect_random_symbol"))
    }

    async fn listen_call(
        &self,
        _req: Request<ParkListenCallRequest>,
    ) -> Result<Response<ParkListenCallResponse>, Status> {
        Err(Status::unimplemented("Park.listen_call"))
    }

    async fn report_action(
        &self,
        _req: Request<ParkReportActionRequest>,
    ) -> Result<Response<ParkReportActionResponse>, Status> {
        Err(Status::unimplemented("Park.report_action"))
    }

    async fn read_talk_free(
        &self,
        _req: Request<ParkReadTalkFreeRequest>,
    ) -> Result<Response<ParkReadTalkFreeResponse>, Status> {
        Err(Status::unimplemented("Park.read_talk_free"))
    }

    async fn select_area(
        &self,
        _req: Request<ParkSelectAreaRequest>,
    ) -> Result<Response<ParkSelectAreaResponse>, Status> {
        Err(Status::unimplemented("Park.select_area"))
    }
}
