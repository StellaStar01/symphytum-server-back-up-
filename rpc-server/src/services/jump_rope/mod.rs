use database::Database;
use rand::RngExt;
use resource::master::{self, MasterTable};
use tonic::{Request, Response, Status};
use types::Validate;
use types::entity::master::{JumpRope as JumpRopeMaster, JumpRopeJumpCountReward, JumpRopeSetting};
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::jump_rope_server::JumpRope;
use types::rpc::api::{
    JumpRopeCreatePrivateRoomRequest, JumpRopeCreatePrivateRoomResponse,
    JumpRopeFinishMultiRequest, JumpRopeFinishMultiResponse, JumpRopeFinishSingleRequest,
    JumpRopeFinishSingleResponse, JumpRopeGetRankingInfoResponse, JumpRopeMatchRandomRequest,
    JumpRopeMatchRandomResponse, JumpRopeReadNotificationRequest, JumpRopeReadNotificationResponse,
    JumpRopeStartMultiRequest, JumpRopeStartMultiResponse, JumpRopeStartSingleRequest,
    JumpRopeStartSingleResponse,
};

mod delta;
mod jump_rope_data;

use crate::auth_token;
use jump_rope_data::{grant_tier_rewards, reward_up_stamina_consumption_quantity};

#[derive(Clone)]
pub struct JumpRopeService {
    db: Database,
}

impl JumpRopeService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<JumpRopeMaster>().await?;
        master::load::<JumpRopeJumpCountReward>().await?;
        master::load::<JumpRopeSetting>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl JumpRope for JumpRopeService {
    async fn match_random(
        &self,
        req: Request<JumpRopeMatchRandomRequest>,
    ) -> Result<Response<JumpRopeMatchRandomResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(JumpRopeMatchRandomResponse::default()))
    }

    async fn create_private_room(
        &self,
        req: Request<JumpRopeCreatePrivateRoomRequest>,
    ) -> Result<Response<JumpRopeCreatePrivateRoomResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(JumpRopeCreatePrivateRoomResponse::default()))
    }

    async fn start_single(
        &self,
        req: Request<JumpRopeStartSingleRequest>,
    ) -> Result<Response<JumpRopeStartSingleResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let rope = JumpRopeMaster::table()
            .iter()
            .find(|r| r.id == req.jump_rope_id)
            .ok_or_else(|| Status::invalid_argument("unknown jump_rope_id"))?;

        // NPC exit counts must be unique (100..199): the client keys them by
        // jump count (ToDictionary) and throws on duplicates.
        let (npc_exits, seed) = {
            let mut rng = rand::rng();
            let count = rope.npc_count.max(0) as usize;
            let mut pool: Vec<i32> = (100..200).collect();
            for i in 0..count.min(pool.len()) {
                let j = rng.random_range(i..pool.len());
                pool.swap(i, j);
            }
            pool.truncate(count);
            (pool, 1 + rng.random_range(1..2_000_000_000))
        };
        let now = database::unix_now();
        self.db
            .start_jump_rope(&uid, &req.jump_rope_id, &npc_exits, now)
            .await
            .map_err(|e| Status::internal(format!("start jump rope: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(JumpRopeStartSingleResponse {
            seed,
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_jump_rope(&updated, &req.jump_rope_id)),
                ..Default::default()
            }),
        }))
    }

    async fn start_multi(
        &self,
        req: Request<JumpRopeStartMultiRequest>,
    ) -> Result<Response<JumpRopeStartMultiResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(JumpRopeStartMultiResponse::default()))
    }

    async fn finish_single(
        &self,
        req: Request<JumpRopeFinishSingleRequest>,
    ) -> Result<Response<JumpRopeFinishSingleResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let now = database::unix_now();

        // the request has no rope id; use the most recently started rope
        let rope_row = self
            .db
            .last_started_jump_rope(&uid)
            .await
            .map_err(|e| Status::internal(format!("in-flight rope: {e}")))?;
        let Some(rope_row) = rope_row else {
            return Err(Status::failed_precondition("no in-flight jump rope"));
        };
        let rope_id = rope_row.jump_rope_id.clone();
        let rope = JumpRopeMaster::table()
            .iter()
            .find(|r| r.id == rope_id)
            .ok_or_else(|| Status::invalid_argument("unknown in-flight rope"))?;

        let past_best = rope_row.best_jump_count as i32;
        let is_best = req.jump_count > past_best;

        let results = grant_tier_rewards(&self.db, &uid, rope, req.jump_count, now)
            .await
            .map_err(|e| Status::internal(format!("grant rewards: {e}")))?;

        // survival semantics: cleared when the run out-jumped every NPC exit
        let npc_exits = self
            .db
            .jump_rope_npc_exits(&uid, &rope_id)
            .await
            .map_err(|e| Status::internal(format!("npc exits: {e}")))?;
        let is_cleared = match npc_exits.iter().map(|e| e.jump_count).min() {
            Some(min_npc) => req.jump_count >= min_npc,
            None => false,
        };
        self.db
            .finish_jump_rope(
                &uid,
                &rope_id,
                (req.jump_count as i64).max(past_best as i64),
                is_cleared,
                now,
            )
            .await
            .map_err(|e| Status::internal(format!("finish jump rope: {e}")))?;

        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(JumpRopeFinishSingleResponse {
            past_best_jump_count: past_best,
            is_best_jump_count_updated: is_best,
            results,
            is_cleared,
            reward_up_stamina_consumption_quantity: reward_up_stamina_consumption_quantity(),
            benefit_license_reward_quantity_up_permil_up: 0,
            reward_ups_by_skill_tree: vec![],
            play_log_upload_url: None,
            gimmick_random_seed: 0,
            marathon_mini_game_result: None,
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_jump_rope(&updated, &rope_id)),
                ..Default::default()
            }),
        }))
    }

    async fn finish_multi(
        &self,
        req: Request<JumpRopeFinishMultiRequest>,
    ) -> Result<Response<JumpRopeFinishMultiResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(JumpRopeFinishMultiResponse::default()))
    }

    async fn get_ranking_info(
        &self,
        _req: Request<()>,
    ) -> Result<Response<JumpRopeGetRankingInfoResponse>, Status> {
        Ok(Response::new(JumpRopeGetRankingInfoResponse::default()))
    }

    async fn read_notification(
        &self,
        req: Request<JumpRopeReadNotificationRequest>,
    ) -> Result<Response<JumpRopeReadNotificationResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(JumpRopeReadNotificationResponse::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn jump_rope_flow_grants_rewards_and_tracks_best() {
        let dir = std::env::temp_dir().join(format!("symjump-{}", std::process::id()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let db = Database::open(&dir.join("test.db")).await.expect("open db");
        let uid = db
            .get_or_create_account("jump-test")
            .await
            .expect("account");
        JumpRopeService::init(db.clone()).await.expect("init");
        // the Exchange and Shop inits load these masters in production
        resource::master::load::<types::entity::master::ExchangeBoothFixedItem>()
            .await
            .expect("fixed items");
        resource::master::load::<types::entity::master::ShopChargeItemProduct>()
            .await
            .expect("charge products");

        let svc = JumpRopeService { db: db.clone() };
        let token = crate::auth_token::mint(&uid);
        let mut start = tonic::Request::new(JumpRopeStartSingleRequest {
            jump_rope_id: "jump_rope-score_attack_001".into(),
            reward_up_stamina_consumption_quantity: 15,
            character_id: "chr-00001".into(),
        });
        start
            .metadata_mut()
            .insert("x-app-auth-token", token.parse().expect("token"));
        let resp = svc.start_single(start).await.expect("start").into_inner();
        assert!(resp.seed > 0, "seed must be positive (Unity InitState)");
        let before = db
            .jump_rope(&uid, "jump_rope-score_attack_001")
            .await
            .expect("row")
            .expect("started rope exists");
        assert_eq!(before.play_count, 1);

        let exits: Vec<i32> = db
            .jump_rope_npc_exits(&uid, "jump_rope-score_attack_001")
            .await
            .expect("npc exits")
            .into_iter()
            .map(|e| e.jump_count)
            .collect();
        assert!(!exits.is_empty(), "score attack has npc exits");
        assert_eq!(
            exits.len(),
            exits.iter().collect::<std::collections::HashSet<_>>().len(),
            "npc exit jump counts must be unique"
        );

        let mut finish = tonic::Request::new(JumpRopeFinishSingleRequest {
            jump_count: 60,
            success_jump_count: 60,
            failure_jump_count: 0,
            continuous_jump_count: 36,
            gimmick_random_seed: 0,
        });
        finish
            .metadata_mut()
            .insert("x-app-auth-token", token.parse().expect("token"));
        let f = svc
            .finish_single(finish)
            .await
            .expect("finish")
            .into_inner();
        assert!(!f.results.is_empty(), "tier rewards for 60 jumps");
        assert_eq!(f.past_best_jump_count, 0);
        assert!(f.is_best_jump_count_updated);
        assert_eq!(
            f.reward_up_stamina_consumption_quantity, 15,
            "JumpRopeSetting unit"
        );

        let row = db
            .jump_rope(&uid, "jump_rope-score_attack_001")
            .await
            .expect("row")
            .expect("rope row exists");
        assert_eq!(row.best_jump_count, 60);
        assert_eq!(row.play_count, 1, "start+finish is one play");
        // the attempt's npc exits are dropped after finish
        assert!(
            db.jump_rope_npc_exits(&uid, "jump_rope-score_attack_001")
                .await
                .expect("exits")
                .is_empty()
        );
    }
}
