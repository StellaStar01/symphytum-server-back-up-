use database::Database;
use resource::master;
use tonic::{Request, Response, Status};
use types::Validate;
use types::entity::master::SkillTreeNode;
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::skill_tree_server::SkillTree;
use types::rpc::api::{
    SkillTreeConnectNodeRequest, SkillTreeConnectNodeResponse, SkillTreeReleaseNodeRequest,
    SkillTreeReleaseNodeResponse, SkillTreeResetNodeRequest, SkillTreeResetNodeResponse,
};

mod delta;

use crate::auth_token;

#[derive(Clone)]
pub struct SkillTreeService {
    db: Database,
}

impl SkillTreeService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<SkillTreeNode>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl SkillTree for SkillTreeService {
    async fn release_node(
        &self,
        req: Request<SkillTreeReleaseNodeRequest>,
    ) -> Result<Response<SkillTreeReleaseNodeResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.db
            .release_skill_tree_node(&uid, &req.character_id, &req.node_group_ids)
            .await
            .map_err(|e| Status::internal(format!("release node: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(SkillTreeReleaseNodeResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_skill_tree(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }

    async fn connect_node(
        &self,
        req: Request<SkillTreeConnectNodeRequest>,
    ) -> Result<Response<SkillTreeConnectNodeResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let released = self
            .db
            .skill_tree_released_groups(&uid, &req.character_id)
            .await
            .map_err(|e| Status::internal(format!("skill tree: {e}")))?;
        if !released.contains(&req.node_group_id) {
            return Err(Status::failed_precondition("node not released"));
        }
        self.db
            .connect_skill_tree_node(&uid, &req.character_id, &req.node_group_id, &req.card_id)
            .await
            .map_err(|e| Status::internal(format!("connect node: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(SkillTreeConnectNodeResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_skill_tree(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }

    async fn reset_node(
        &self,
        req: Request<SkillTreeResetNodeRequest>,
    ) -> Result<Response<SkillTreeResetNodeResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.db
            .reset_skill_tree_node(&uid, &req.character_id, &req.node_group_ids)
            .await
            .map_err(|e| Status::internal(format!("reset node: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(SkillTreeResetNodeResponse {
            reward_results: vec![],
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_skill_tree(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }
}
