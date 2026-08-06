use database::Database;
use tonic::{Request, Response, Status};

use resource::master::{self, MasterTable};
use types::Validate;
use types::entity::master::{
    Card, CardLevel, Character, Costume, LiveDeckEvaluationRank, LiveDeckPowerRank,
    LiveLeaderSkill, LivePassiveSkillEffect, LiveScoreEvaluationRank, Music,
};
use types::enums::LiveResultType;
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::live_get_deck_candidate_card_parameters_response::ParameterInfo;
use types::rpc::api::live_server::Live;
use types::rpc::api::{
    LiveContinueRequest, LiveContinueResponse, LiveCreatePrivateRoomRequest,
    LiveCreatePrivateRoomResponse, LiveFinishDisconnectedMultiLiveCooperationRequest,
    LiveFinishDisconnectedMultiLiveCooperationResponse, LiveFinishMultiLiveCooperationRequest,
    LiveFinishMultiLiveCooperationResponse, LiveFinishSingleRequest, LiveFinishSingleResponse,
    LiveGetDeckCandidateCardParametersRequest, LiveGetDeckCandidateCardParametersResponse,
    LiveGetDeckRequest, LiveGetDeckResponse, LiveGetDraftDeckInfoRequest,
    LiveGetDraftDeckInfoResponse, LiveListRecommendCharacterDeckRequest,
    LiveListRecommendCharacterDeckResponse, LiveMatchRandomRequest, LiveMatchRandomResponse,
    LiveRecoverRewardUpStaminaRequest, LiveRecoverRewardUpStaminaResponse, LiveRetireRequest,
    LiveRetireResponse, LiveRetryRequest, LiveRetryResponse, LiveSaveDeckRequest,
    LiveSaveDeckResponse, LiveSetRewardUpStaminaConsumptionSettingRequest,
    LiveSetRewardUpStaminaConsumptionSettingResponse, LiveSetSkinLogRequest,
    LiveSetSkinLogResponse, LiveStartMultiRequest, LiveStartMultiResponse, LiveStartSingleRequest,
    LiveStartSingleResponse,
};

mod delta;
mod live_data;

#[derive(Clone)]
pub struct LiveService {
    db: Database,
}

impl LiveService {
    // Unity's Random.InitState rejects seeds < 1
    const fn seed() -> i32 {
        // (rand::random::<u32>() % 2_000_000_000 + 1) as i32;
        1
    }

    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<Card>().await?;
        master::load::<CardLevel>().await?;
        master::load::<Character>().await?;
        master::load::<Costume>().await?;
        master::load::<LiveDeckEvaluationRank>().await?;
        master::load::<LiveDeckPowerRank>().await?;
        master::load::<LiveLeaderSkill>().await?;
        master::load::<LivePassiveSkillEffect>().await?;
        master::load::<LiveScoreEvaluationRank>().await?;
        master::load::<Music>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Live for LiveService {
    async fn get_deck_candidate_card_parameters(
        &self,
        req: Request<LiveGetDeckCandidateCardParametersRequest>,
    ) -> Result<Response<LiveGetDeckCandidateCardParametersResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let infos = Card::table()
            .iter()
            .filter(|c| c.character_id == req.character_id)
            .map(|card| {
                let (perf, tech, sense) = live_data::card_stats(card);
                ParameterInfo {
                    card_id: card.id.clone(),
                    parameter: perf + tech + sense,
                    performance: perf,
                    technique: tech,
                    sense,
                }
            })
            .collect();
        Ok(Response::new(LiveGetDeckCandidateCardParametersResponse {
            parameter_infos: infos,
            common_response: None,
        }))
    }

    async fn get_draft_deck_info(
        &self,
        req: Request<LiveGetDraftDeckInfoRequest>,
    ) -> Result<Response<LiveGetDraftDeckInfoResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let cards: Vec<(i32, String)> = req
            .deck_positions
            .iter()
            .map(|p| (p.position, p.card_id.clone()))
            .collect();
        let resolved = live_data::resolve_positions(&cards);
        let total: i64 = resolved.iter().map(|(_, _, p, _, _, _)| p).sum();
        Ok(Response::new(LiveGetDraftDeckInfoResponse {
            live_deck_power: total,
            deck_positions: live_data::deck_positions(&resolved),
            common_response: None,
        }))
    }

    async fn get_deck(
        &self,
        req: Request<LiveGetDeckRequest>,
    ) -> Result<Response<LiveGetDeckResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let deck = self
            .db
            .get_deck(&uid, &req.character_id, req.number)
            .await
            .map_err(|e| Status::internal(format!("get deck: {e}")))?;
        let positions = deck
            .as_ref()
            .map(|d| d.positions.clone())
            .unwrap_or_default();
        let resolved = live_data::resolve_positions_for_user(&self.db, &uid, &positions)
            .await
            .map_err(|e| Status::internal(format!("resolve positions: {e}")))?;
        let member_parameter: i64 = resolved.iter().map(|r| r.6).sum();
        let members: Vec<(&Card, i64)> = resolved
            .iter()
            .filter_map(|(_, cid, _, _, _, _, base)| {
                Card::table()
                    .iter()
                    .find(|c| &c.id == cid)
                    .map(|c| (c, *base))
            })
            .collect();
        let outfit_up = live_data::outfit_skill_up(
            &deck
                .as_ref()
                .map(|d| d.costume_id.clone())
                .unwrap_or_default(),
            &members,
        );
        let power_rows: Vec<(i32, String, i64)> = resolved
            .iter()
            .map(|(p, c, pw, _, _, _, _)| (*p, c.clone(), *pw))
            .collect();
        let resolved6: Vec<(i32, String, i64, i64, i64, i64)> = resolved
            .iter()
            .map(|(p, c, pw, perf, tech, sense, _)| (*p, c.clone(), *pw, *perf, *tech, *sense))
            .collect();
        Ok(Response::new(LiveGetDeckResponse {
            deck_positions: live_data::deck_positions(&resolved6),
            live_deck_evaluation: live_data::evaluation_for(
                &power_rows,
                member_parameter,
                outfit_up,
            ),
            live_deck_in_game_effect: live_data::in_game_effect(&resolved6),
            common_response: None,
        }))
    }

    async fn save_deck(
        &self,
        req: Request<LiveSaveDeckRequest>,
    ) -> Result<Response<LiveSaveDeckResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cards: Vec<(i32, String)> = req
            .deck_positions
            .iter()
            .map(|p| (p.position, p.card_id.clone()))
            .collect();
        self.db
            .save_deck(
                &uid,
                &req.character_id,
                req.number,
                &req.name,
                &req.costume_id,
                &cards,
            )
            .await
            .map_err(|e| Status::internal(format!("save deck: {e}")))?;
        let resolved = live_data::resolve_positions_for_user(&self.db, &uid, &cards)
            .await
            .map_err(|e| Status::internal(format!("resolve positions: {e}")))?;
        let member_parameter: i64 = resolved.iter().map(|r| r.6).sum();
        let members: Vec<(&Card, i64)> = resolved
            .iter()
            .filter_map(|(_, cid, _, _, _, _, base)| {
                Card::table()
                    .iter()
                    .find(|c| &c.id == cid)
                    .map(|c| (c, *base))
            })
            .collect();
        let outfit_up = live_data::outfit_skill_up(&req.costume_id, &members);
        let power_rows: Vec<(i32, String, i64)> = resolved
            .iter()
            .map(|(p, c, pw, _, _, _, _)| (*p, c.clone(), *pw))
            .collect();
        let resolved6: Vec<(i32, String, i64, i64, i64, i64)> = resolved
            .iter()
            .map(|(p, c, pw, perf, tech, sense, _)| (*p, c.clone(), *pw, *perf, *tech, *sense))
            .collect();
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(LiveSaveDeckResponse {
            live_deck_evaluation: live_data::evaluation_for(
                &power_rows,
                member_parameter,
                outfit_up,
            ),
            live_deck_in_game_effect: live_data::in_game_effect(&resolved6),
            deck_positions: live_data::deck_positions(&resolved6),
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_save_deck(
                    &updated,
                    &req.character_id,
                    req.number,
                )),
                ..Default::default()
            }),
        }))
    }

    async fn set_skin_log(
        &self,
        req: Request<LiveSetSkinLogRequest>,
    ) -> Result<Response<LiveSetSkinLogResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(LiveSetSkinLogResponse::default()))
    }

    async fn set_reward_up_stamina_consumption_setting(
        &self,
        req: Request<LiveSetRewardUpStaminaConsumptionSettingRequest>,
    ) -> Result<Response<LiveSetRewardUpStaminaConsumptionSettingResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let mut live = self
            .db
            .get_live(&uid)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        live.reward_up_stamina_consumption_setting_quantity = req.quantity;
        self.db
            .save_live(&live)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(
            LiveSetRewardUpStaminaConsumptionSettingResponse::default(),
        ))
    }

    async fn recover_reward_up_stamina(
        &self,
        req: Request<LiveRecoverRewardUpStaminaRequest>,
    ) -> Result<Response<LiveRecoverRewardUpStaminaResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        for c in &req.consumptions {
            self.db
                .consume_item(&uid, &c.resource_id, c.quantity)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }
        Ok(Response::new(LiveRecoverRewardUpStaminaResponse::default()))
    }

    async fn match_random(
        &self,
        _req: Request<LiveMatchRandomRequest>,
    ) -> Result<Response<LiveMatchRandomResponse>, Status> {
        // multi-game ignored for now
        Ok(Response::new(LiveMatchRandomResponse::default()))
    }

    async fn create_private_room(
        &self,
        _req: Request<LiveCreatePrivateRoomRequest>,
    ) -> Result<Response<LiveCreatePrivateRoomResponse>, Status> {
        Ok(Response::new(LiveCreatePrivateRoomResponse::default()))
    }

    async fn start_single(
        &self,
        req: Request<LiveStartSingleRequest>,
    ) -> Result<Response<LiveStartSingleResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // remember the in-flight single so FinishSingle knows the context
        let mut live = self
            .db
            .get_live(&uid)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        live.last_single_played_music_id = req.music_id.clone();
        live.last_single_played_music_difficulty_type = req.music_difficulty_type;
        live.last_played_character_id = req.character_id.clone();
        live.reward_up_stamina_consumption_setting_quantity =
            req.reward_up_stamina_consumption_quantity as i64;
        self.db
            .save_live(&live)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(LiveStartSingleResponse {
            seed: Self::seed(),
            common_response: None,
        }))
    }

    async fn start_multi(
        &self,
        _req: Request<LiveStartMultiRequest>,
    ) -> Result<Response<LiveStartMultiResponse>, Status> {
        Ok(Response::new(LiveStartMultiResponse {
            seed: Self::seed(),
            common_response: None,
        }))
    }

    async fn finish_single(
        &self,
        req: Request<LiveFinishSingleRequest>,
    ) -> Result<Response<LiveFinishSingleResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let live = self
            .db
            .get_live(&uid)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let result = req
            .result
            .ok_or_else(|| Status::invalid_argument("missing result"))?;

        let all_perfect = result.miss_count == 0
            && result.bad_count == 0
            && result.good_count == 0
            && result.great_count == 0;
        let full_combo = result.miss_count == 0 && result.bad_count == 0;
        let cleared = result.remaining_life > 0 || result.miss_count == 0;
        let result_type = if all_perfect {
            LiveResultType::AllPerfect
        } else if full_combo {
            LiveResultType::FullCombo
        } else if cleared {
            LiveResultType::Cleared
        } else {
            LiveResultType::NotCleared
        };

        // past highest before recording
        let past = self
            .db
            .past_character_highest_score(
                &uid,
                &live.last_single_played_music_id,
                live.last_single_played_music_difficulty_type,
            )
            .await
            .map_err(|e| Status::internal(format!("past score: {e}")))?;

        self.db
            .record_single_result(
                &uid,
                &live.last_single_played_music_id,
                live.last_single_played_music_difficulty_type,
                &live.last_played_character_id,
                "",
                result.score,
                result.max_combo_count as i64,
                result_type != LiveResultType::NotCleared,
                result_type as i32,
            )
            .await
            .map_err(|e| Status::internal(format!("record: {e}")))?;

        let (evaluation_rank_type, evaluation_rank_plus) =
            live_data::score_evaluation_rank(&live.last_single_played_music_id, result.score);

        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;

        Ok(Response::new(LiveFinishSingleResponse {
            result_type: result_type as i32,
            evaluation_rank_type: evaluation_rank_type as i32,
            evaluation_rank_plus: evaluation_rank_plus,
            past_character_highest_score: past,
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_finish_live(
                    &updated,
                    &live.last_single_played_music_id,
                    &live.last_played_character_id,
                )),
                ..Default::default()
            }),
            ..Default::default()
        }))
    }

    async fn finish_multi_live_cooperation(
        &self,
        _req: Request<LiveFinishMultiLiveCooperationRequest>,
    ) -> Result<Response<LiveFinishMultiLiveCooperationResponse>, Status> {
        Ok(Response::new(
            LiveFinishMultiLiveCooperationResponse::default(),
        ))
    }

    async fn finish_disconnected_multi_live_cooperation(
        &self,
        _req: Request<LiveFinishDisconnectedMultiLiveCooperationRequest>,
    ) -> Result<Response<LiveFinishDisconnectedMultiLiveCooperationResponse>, Status> {
        Ok(Response::new(
            LiveFinishDisconnectedMultiLiveCooperationResponse::default(),
        ))
    }

    async fn r#continue(
        &self,
        _req: Request<LiveContinueRequest>,
    ) -> Result<Response<LiveContinueResponse>, Status> {
        Ok(Response::new(LiveContinueResponse::default()))
    }

    async fn retry(
        &self,
        _req: Request<LiveRetryRequest>,
    ) -> Result<Response<LiveRetryResponse>, Status> {
        Ok(Response::new(LiveRetryResponse {
            seed: Self::seed(),
            common_response: None,
        }))
    }

    async fn retire(
        &self,
        _req: Request<LiveRetireRequest>,
    ) -> Result<Response<LiveRetireResponse>, Status> {
        Ok(Response::new(LiveRetireResponse::default()))
    }

    async fn list_recommend_character_deck(
        &self,
        _req: Request<LiveListRecommendCharacterDeckRequest>,
    ) -> Result<Response<LiveListRecommendCharacterDeckResponse>, Status> {
        Ok(Response::new(
            LiveListRecommendCharacterDeckResponse::default(),
        ))
    }
}
