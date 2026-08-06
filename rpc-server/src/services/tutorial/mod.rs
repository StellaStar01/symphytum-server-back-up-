use database::Database;
use tonic::{Request, Response, Status};

use resource::master::{self, MasterTable};
use types::Validate;
use types::entity::master::Card;
use types::rpc::api::tutorial_server::Tutorial;
use types::rpc::api::{
    TutorialChooseCharacterRequest, TutorialChooseCharacterResponse, TutorialConfirmGachaResponse,
    TutorialDrawGachaResponse, TutorialListGachaCardResponse, TutorialProgressRequest,
    TutorialProgressResponse, TutorialReadInstantTipsRequest, TutorialReadInstantTipsResponse,
    TutorialRegisterInitialUserInfoRequest, TutorialRegisterInitialUserInfoResponse,
};

use crate::auth_token;

#[derive(Clone)]
pub struct TutorialService {
    db: Database,
}

impl TutorialService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<Card>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Tutorial for TutorialService {
    async fn progress(
        &self,
        req: Request<TutorialProgressRequest>,
    ) -> Result<Response<TutorialProgressResponse>, Status> {
        let uid_opt = auth_token::uid_opt(&req);
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if let Some(uid) = uid_opt {
            self.db
                .set_tutorial_step(&uid, req.tutorial_type, req.progress)
                .await
                .map_err(|e| Status::internal(format!("tutorial: {e}")))?;
        }
        Ok(Response::new(TutorialProgressResponse::default()))
    }

    async fn register_initial_user_info(
        &self,
        req: Request<TutorialRegisterInitialUserInfoRequest>,
    ) -> Result<Response<TutorialRegisterInitialUserInfoResponse>, Status> {
        let uid_opt = auth_token::uid_opt(&req);
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if let Some(uid) = uid_opt {
            self.db
                .set_initial_user_info(
                    &uid,
                    &req.country_code,
                    req.birth_year,
                    req.birth_month,
                    &req.user_name,
                )
                .await
                .map_err(|e| Status::internal(format!("tutorial user: {e}")))?;
        }
        Ok(Response::new(
            TutorialRegisterInitialUserInfoResponse::default(),
        ))
    }

    async fn choose_character(
        &self,
        req: Request<TutorialChooseCharacterRequest>,
    ) -> Result<Response<TutorialChooseCharacterResponse>, Status> {
        let uid_opt = auth_token::uid_opt(&req);
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if let Some(uid) = uid_opt {
            self.db
                .set_park_character(&uid, &req.character_id)
                .await
                .map_err(|e| Status::internal(format!("tutorial character: {e}")))?;
        }
        Ok(Response::new(TutorialChooseCharacterResponse::default()))
    }

    async fn draw_gacha(
        &self,
        _req: Request<()>,
    ) -> Result<Response<TutorialDrawGachaResponse>, Status> {
        let card_ids = Card::table().iter().take(3).map(|c| c.id.clone()).collect();
        Ok(Response::new(TutorialDrawGachaResponse {
            card_ids,
            gacha_animation_grouping_id: "gacha_animation_grouping-default-001".into(),
            common_response: None,
        }))
    }

    async fn confirm_gacha(
        &self,
        _req: Request<()>,
    ) -> Result<Response<TutorialConfirmGachaResponse>, Status> {
        Ok(Response::new(TutorialConfirmGachaResponse::default()))
    }

    async fn list_gacha_card(
        &self,
        _req: Request<()>,
    ) -> Result<Response<TutorialListGachaCardResponse>, Status> {
        let card_ids = Card::table().iter().take(3).map(|c| c.id.clone()).collect();
        Ok(Response::new(TutorialListGachaCardResponse {
            card_ids,
            common_response: None,
        }))
    }

    async fn read_instant_tips(
        &self,
        req: Request<TutorialReadInstantTipsRequest>,
    ) -> Result<Response<TutorialReadInstantTipsResponse>, Status> {
        let uid_opt = auth_token::uid_opt(&req);
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if let Some(uid) = uid_opt {
            self.db
                .read_instant_tip(&uid, &req.instant_tips_id)
                .await
                .map_err(|e| Status::internal(format!("tips: {e}")))?;
        }
        Ok(Response::new(TutorialReadInstantTipsResponse::default()))
    }
}
