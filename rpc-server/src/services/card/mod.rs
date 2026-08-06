use database::Database;
use resource::master::{self, MasterTable};
use tonic::{Request, Response, Status};
use types::Validate;
use types::entity::master::{
    Card as CardMaster, CardLevel, CardLevelLimit, CardPotential, SkillTreeConnectEffect,
    SkillTreeEffect, SkillTreeNode,
};
use types::rpc::api::card_get_parameter_response::SkillTreeEffect as GetParamSkillTreeEffect;
use types::rpc::api::card_get_parameters_response::ParameterInfo;
use types::rpc::api::card_server::Card;
use types::rpc::api::{
    CardGetParameterRequest, CardGetParameterResponse, CardGetParametersResponse,
    CardLevelLimitBreakRequest, CardLevelLimitBreakResponse, CardLevelUpRequest,
    CardLevelUpResponse, CardUpgradePotentialRequest, CardUpgradePotentialResponse,
};

use crate::auth_token;

mod card_data;

pub struct CardService {
    db: Database,
}

impl CardService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<CardMaster>().await?;
        master::load::<CardLevel>().await?;
        master::load::<CardLevelLimit>().await?;
        master::load::<CardPotential>().await?;
        master::load::<SkillTreeNode>().await?;
        master::load::<SkillTreeEffect>().await?;
        master::load::<SkillTreeConnectEffect>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Card for CardService {
    async fn get_parameter(
        &self,
        req: Request<CardGetParameterRequest>,
    ) -> Result<Response<CardGetParameterResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let Some(card) = CardMaster::table().iter().find(|c| c.id == req.card_id) else {
            return Err(Status::not_found(format!("unknown card: {}", req.card_id)));
        };
        let Some(uc) = self
            .db
            .user_card(&uid, &req.card_id)
            .await
            .map_err(|e| Status::internal(format!("user card: {e}")))?
        else {
            return Err(Status::not_found(format!(
                "card not owned: {}",
                req.card_id
            )));
        };

        let level = card_data::level_of(&card, uc.exp, uc.level_limit_break_count);
        let parameter = card_data::parameter_base(&card, level);
        let (bonus, connect_level) = card_data::potential_bonus(&card, uc.potential_upgrade_count);
        let performance =
            card_data::attribute_value(parameter, card.performance_permil_multiply, bonus);
        let technique =
            card_data::attribute_value(parameter, card.technique_permil_multiply, bonus);
        let sense = card_data::attribute_value(parameter, card.sense_permil_multiply, bonus);

        let released = self
            .db
            .skill_tree_released_groups(&uid, &card.character_id)
            .await
            .map_err(|e| Status::internal(format!("skill tree: {e}")))?;
        let (p, t, s, pp, tp, sp) = card_data::skill_tree_bonus(&card, &released, connect_level);

        Ok(Response::new(CardGetParameterResponse {
            parameter,
            performance,
            technique,
            sense,
            skill_tree_effect: Some(GetParamSkillTreeEffect {
                performance_up: p as i32,
                performance_up_permil_up: pp as f32 / 1000.0,
                technique_up: t as i32,
                technique_up_permil_up: tp as f32 / 1000.0,
                sense_up: s as i32,
                sense_up_permil_up: sp as f32 / 1000.0,
            }),
            common_response: None,
        }))
    }

    async fn level_up(
        &self,
        _req: Request<CardLevelUpRequest>,
    ) -> Result<Response<CardLevelUpResponse>, Status> {
        Err(Status::unimplemented("Card.level_up"))
    }

    async fn level_limit_break(
        &self,
        _req: Request<CardLevelLimitBreakRequest>,
    ) -> Result<Response<CardLevelLimitBreakResponse>, Status> {
        Err(Status::unimplemented("Card.level_limit_break"))
    }

    async fn upgrade_potential(
        &self,
        _req: Request<CardUpgradePotentialRequest>,
    ) -> Result<Response<CardUpgradePotentialResponse>, Status> {
        Err(Status::unimplemented("Card.upgrade_potential"))
    }

    async fn get_parameters(
        &self,
        req: Request<()>,
    ) -> Result<Response<CardGetParametersResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let cards = self
            .db
            .user_cards(&uid)
            .await
            .map_err(|e| Status::internal(format!("user cards: {e}")))?;
        let mut parameter_infos = Vec::with_capacity(cards.len());
        let mut deck_power_permil = 0i64;
        for uc in cards {
            let Some(card) = CardMaster::table().iter().find(|c| c.id == uc.card_id) else {
                continue;
            };
            let level = card_data::level_of(&card, uc.exp, uc.level_limit_break_count);
            if let Some(l) = CardLevel::table()
                .iter()
                .find(|l| l.group_id == card.card_level_group_id && l.level == level)
            {
                deck_power_permil = deck_power_permil.max(l.live_deck_power_permyriad_up);
            }
            let Some((parameter, performance, technique, sense)) =
                card_data::parameters_for(&self.db, &uid, &uc.card_id)
                    .await
                    .map_err(|e| Status::internal(format!("parameters: {e}")))?
            else {
                continue;
            };
            parameter_infos.push(ParameterInfo {
                card_id: uc.card_id,
                parameter,
                performance,
                technique,
                sense,
            });
        }
        Ok(Response::new(CardGetParametersResponse {
            parameter_infos,
            live_deck_power_permyriad_up_by_card_level_up: deck_power_permil as i32,
            common_response: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::Database;

    async fn temp_db(tag: &str) -> Database {
        // unique per run: a reused pid leaves stale files behind
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("cardtest-{}-{}-{}", std::process::id(), nanos, tag));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let path = dir.join("test.db");
        match Database::open(&path).await {
            Ok(db) => db,
            Err(e) => panic!("open db {path:?}: {e}"),
        }
    }

    #[tokio::test]
    async fn parameter_matches_client_formula() {
        let db = temp_db("a").await;
        // get_or_create_account maxes; reset one card to level-1 state
        let uid = db
            .get_or_create_account("card-test")
            .await
            .expect("account");
        let card_id = "card-00012-5-uniq-0062-00";
        // rarity_5 card, level group grp-card-level-rarity_5-02, lbc 0 -> limit 40
        sqlx::query("UPDATE user_cards SET exp = 0, level_limit_break_count = 0, potential_upgrade_count = 0 WHERE account_id = ? AND card_id = ?")
            .bind(&uid).bind(card_id)
            .execute(db.pool()).await.expect("reset card");

        let svc = CardService::init(db).await.expect("init");
        let token = crate::auth_token::mint(&uid);
        let mut req = tonic::Request::new(CardGetParameterRequest {
            card_id: card_id.into(),
        });
        req.metadata_mut()
            .insert("x-app-auth-token", token.parse().expect("token"));
        let resp = svc
            .get_parameter(req)
            .await
            .expect("get_parameter")
            .into_inner();

        // level 1 -> parameter 5269; no potential -> bonus 0, connect 1
        assert_eq!(resp.parameter, 5269, "level-1 base parameter");
        // ceil(permil * 5269/1000 * 1.0) in f32
        assert_eq!(resp.performance, 1613);
        assert_eq!(resp.technique, 1550);
        assert_eq!(resp.sense, 2108);
        let tree = resp.skill_tree_effect.expect("tree effect");
        // maxed account: all nodes released, so flat ups and the connect permil are non-zero
        assert!(tree.performance_up > 0);
        assert!(tree.technique_up > 0);
        assert!(tree.sense_up > 0);
        assert!(tree.performance_up_permil_up > 0.0);
    }

    #[tokio::test]
    async fn rejects_unknown_or_unowned_card() {
        let db = temp_db("a").await;
        let uid = db
            .get_or_create_account("card-test-2")
            .await
            .expect("account");
        let _ = uid;
        let svc = CardService::init(db).await.expect("init");
        let token = crate::auth_token::mint(&uid);
        let mut req = tonic::Request::new(CardGetParameterRequest {
            card_id: "card-nope".into(),
        });
        req.metadata_mut()
            .insert("x-app-auth-token", token.parse().expect("token"));
        assert!(svc.get_parameter(req).await.is_err());
    }
}
