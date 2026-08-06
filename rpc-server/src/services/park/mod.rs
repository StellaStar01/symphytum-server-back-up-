use database::Database;
use resource::master;
use tonic::{Request, Response, Status};
use types::Validate;

use types::entity::master::PlayerLevel;
use types::entity::transaction::UserPark;
use types::rpc::api::common::{Response as CommonResponse, UserData};
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

mod park_data;
mod player_level_rewards;

use crate::auth_token;

#[derive(Clone)]
pub struct ParkService {
    db: Database,
}

impl ParkService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<PlayerLevel>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Park for ParkService {
    async fn enter(&self, req: Request<()>) -> Result<Response<ParkEnterResponse>, Status> {
        // cant be derived from master data, how sad!
        let mut resp = park_data::response();
        if let Some(uid) = auth_token::uid_opt(&req) {
            let data = self
                .db
                .user_data(&uid)
                .await
                .map_err(|e| Status::internal(format!("load user: {e}")))?;
            let mut updated = UserData::default();
            updated.user_park = data.user_park.clone();
            resp.common_response = Some(CommonResponse {
                updated_data: Some(updated),
                ..Default::default()
            });
        }
        Ok(Response::new(resp))
    }

    async fn refresh(
        &self,
        req: Request<ParkRefreshRequest>,
    ) -> Result<Response<ParkRefreshResponse>, Status> {
        let mut resp = ParkRefreshResponse::default();
        if let Some(uid) = auth_token::uid_opt(&req) {
            let data = self
                .db
                .user_data(&uid)
                .await
                .map_err(|e| Status::internal(format!("load user: {e}")))?;
            let mut updated = UserData::default();
            updated.user_park = data.user_park.clone();
            resp.common_response = Some(CommonResponse {
                updated_data: Some(updated),
                ..Default::default()
            });
        }
        Ok(Response::new(resp))
    }
    async fn set_character(
        &self,
        req: Request<ParkSetCharacterRequest>,
    ) -> Result<Response<ParkSetCharacterResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        // only owned characters can stand in the park
        if !self
            .db
            .owns_character(&uid, &req.character_id)
            .await
            .map_err(|e| Status::internal(format!("character: {e}")))?
        {
            return Err(Status::invalid_argument("character not owned"));
        }
        self.db
            .set_park_character(&uid, &req.character_id)
            .await
            .map_err(|e| Status::internal(format!("set park character: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        let mut d = UserData::default();
        // the client resolves the overworld member from user_profile.park_character_id
        d.user_profile = updated.user_profile.clone();
        Ok(Response::new(ParkSetCharacterResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(d),
                ..Default::default()
            }),
        }))
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
        // day/night toggle; no user state to persist
        Ok(Response::new(ParkUpdateTimePeriodResponse::default()))
    }

    async fn receive_player_level_reward(
        &self,
        req: Request<ParkReceivePlayerLevelRewardRequest>,
    ) -> Result<Response<ParkReceivePlayerLevelRewardResponse>, Status> {
        let uid = auth_token::uid_opt(&req);
        let req = req.into_inner();
        let Some(uid) = uid else {
            return Ok(Response::new(ParkReceivePlayerLevelRewardResponse {
                reward_results: Vec::new(),
                ..Default::default()
            }));
        };

        let (reward_results, received) =
            player_level_rewards::player_level_rewards(&self.db, &uid, &req.player_levels).await?;

        let mut updated = UserData::default();
        updated.user_park = Some(UserPark {
            received_player_level_reward_levels: received,
            ..Default::default()
        });
        Ok(Response::new(ParkReceivePlayerLevelRewardResponse {
            reward_results,
            common_response: Some(CommonResponse {
                updated_data: Some(updated),
                ..Default::default()
            }),
        }))
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
