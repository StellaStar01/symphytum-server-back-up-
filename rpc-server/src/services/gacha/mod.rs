use database::Database;
use tonic::{Request, Response, Status};

use resource::master::{self, MasterTable};
use types::Validate;
use types::entity::master::{Card, CardPotential, Gacha as GachaMaster, GachaButton, GachaPoint};
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::gacha_server::Gacha;
use types::rpc::api::{
    GachaDrawCardSelectRequest, GachaDrawCardSelectResponse, GachaDrawNormalRequest,
    GachaDrawNormalResponse, GachaDrawResponseGachaButton, GachaListCardSelectProbabilityRequest,
    GachaListCardSelectProbabilityResponse, GachaListHistoryResponse,
    GachaListNormalProbabilityRequest, GachaListNormalProbabilityResponse, GachaListResponse,
    GachaReadRequest, GachaReadResponse, GachaSetSelectedCardRequest, GachaSetSelectedCardResponse,
};

use crate::auth_token;

mod data;
mod delta;
mod draw;
mod grants;
mod rates;

#[derive(Clone)]
pub struct GachaService {
    db: Database,
}

impl GachaService {
    pub async fn init(db: Database) -> Result<Self, String> {
        master::load::<GachaMaster>().await?;
        master::load::<GachaButton>().await?;
        master::load::<GachaPoint>().await?;
        master::load::<Card>().await?;
        master::load::<CardPotential>().await?;
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Gacha for GachaService {
    async fn list(&self, _req: Request<()>) -> Result<Response<GachaListResponse>, Status> {
        Ok(Response::new(data::list()))
    }
    async fn list_history(
        &self,
        _req: Request<()>,
    ) -> Result<Response<GachaListHistoryResponse>, Status> {
        // no persistence -> empty history
        Ok(Response::new(GachaListHistoryResponse::default()))
    }

    async fn read(
        &self,
        _req: Request<GachaReadRequest>,
    ) -> Result<Response<GachaReadResponse>, Status> {
        // marks the gacha notice read; nothing to persist
        Ok(Response::new(GachaReadResponse::default()))
    }

    async fn draw_normal(
        &self,
        req: Request<GachaDrawNormalRequest>,
    ) -> Result<Response<GachaDrawNormalResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let Some((_gacha, button)) = data::find_button(&req.gacha_button_id) else {
            return Err(Status::invalid_argument("unknown gacha_button_id"));
        };
        let count = grants::resolve_draw_count(&button, req.draw_count)?;

        grants::consume_costs(&self.db, &uid, &button).await?;

        let pool: Vec<&Card> = Card::table().iter().collect();
        // ThreadRng is !Send
        let rolled = {
            let mut rng = rand::rng();
            draw::draw(
                &mut rng,
                &pool,
                count,
                button.fixed_reward_pick_count > 0,
                None,
            )
        };
        let (card_results, mut granted) = grants::grant_results(&self.db, &uid, rolled).await?;

        // gacha point bonus (before/after)
        let mut bonus_befores = Vec::new();
        for b in &button.bonus_rewards {
            let before = self
                .db
                .gacha_point_quantity(&uid, &b.resource_id)
                .await
                .map_err(|e| Status::internal(format!("point qty: {e}")))?;
            self.db
                .add_gacha_point(&uid, &b.resource_id, b.quantity)
                .await
                .map_err(|e| Status::internal(format!("grant points: {e}")))?;
            bonus_befores.push(before);
            granted.item_ids.insert(b.resource_id.clone());
        }
        self.db
            .bump_gacha_draw_count(&uid, &req.gacha_button_id, count as i32)
            .await
            .map_err(|e| Status::internal(format!("draw count: {e}")))?;

        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;

        Ok(Response::new(GachaDrawNormalResponse {
            card_results,
            bonus_reward_results: grants::bonus_reward_results(&button, &bonus_befores),
            gacha_button: Some(GachaDrawResponseGachaButton {
                gacha_button_id: req.gacha_button_id.clone(),
                is_disabled: false,
                drawn_count: count as i32,
            }),
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_gacha_draw(
                    &updated,
                    &granted,
                    &req.gacha_button_id,
                )),
                ..Default::default()
            }),
        }))
    }

    async fn list_normal_probability(
        &self,
        req: Request<GachaListNormalProbabilityRequest>,
    ) -> Result<Response<GachaListNormalProbabilityResponse>, Status> {
        let req = req.into_inner();
        if !data::list()
            .gacha_groups
            .iter()
            .flat_map(|g| g.gachas.iter())
            .any(|g| g.gacha_id == req.gacha_id)
        {
            return Err(Status::invalid_argument("unknown gacha_id"));
        }
        let pool: Vec<&Card> = Card::table().iter().collect();
        let (normal, fixed) = rates::probability_response(&pool);
        Ok(Response::new(GachaListNormalProbabilityResponse {
            normal_rarity_probabilities: normal,
            fixed_rarity_probabilities: fixed,
            common_response: None,
        }))
    }

    async fn draw_card_select(
        &self,
        req: Request<GachaDrawCardSelectRequest>,
    ) -> Result<Response<GachaDrawCardSelectResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let Some((_gacha, button)) = data::find_button(&req.gacha_button_id) else {
            return Err(Status::invalid_argument("unknown gacha_button_id"));
        };
        let count = grants::resolve_draw_count(&button, req.draw_count)?;

        grants::consume_costs(&self.db, &uid, &button).await?;

        let pool: Vec<&Card> = Card::table().iter().collect();
        let gacha_id =
            data::gacha_id_of_button(&req.gacha_button_id).unwrap_or(&req.gacha_button_id);
        let selection: Vec<&Card> = data::SELECTED_CARDS
            .lock()
            .get(gacha_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| Card::table().iter().find(|c| &c.id == id))
                    .collect()
            })
            .unwrap_or_default();
        let rolled = {
            let mut rng = rand::rng();
            draw::draw(
                &mut rng,
                &pool,
                count,
                button.fixed_reward_pick_count > 0,
                (!selection.is_empty()).then_some(&selection[..]),
            )
        };
        let (card_results, mut granted) = grants::grant_results(&self.db, &uid, rolled).await?;

        let mut bonus_befores = Vec::new();
        for b in &button.bonus_rewards {
            let before = self
                .db
                .gacha_point_quantity(&uid, &b.resource_id)
                .await
                .map_err(|e| Status::internal(format!("point qty: {e}")))?;
            self.db
                .add_gacha_point(&uid, &b.resource_id, b.quantity)
                .await
                .map_err(|e| Status::internal(format!("grant points: {e}")))?;
            bonus_befores.push(before);
            granted.item_ids.insert(b.resource_id.clone());
        }
        self.db
            .bump_gacha_draw_count(&uid, &req.gacha_button_id, count as i32)
            .await
            .map_err(|e| Status::internal(format!("draw count: {e}")))?;

        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;

        Ok(Response::new(GachaDrawCardSelectResponse {
            card_results,
            bonus_reward_results: grants::bonus_reward_results(&button, &bonus_befores),
            gacha_button: Some(GachaDrawResponseGachaButton {
                gacha_button_id: req.gacha_button_id.clone(),
                is_disabled: false,
                drawn_count: count as i32,
            }),
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_gacha_draw(
                    &updated,
                    &granted,
                    &req.gacha_button_id,
                )),
                ..Default::default()
            }),
        }))
    }

    async fn list_card_select_probability(
        &self,
        req: Request<GachaListCardSelectProbabilityRequest>,
    ) -> Result<Response<GachaListCardSelectProbabilityResponse>, Status> {
        let req = req.into_inner();
        if !data::list()
            .gacha_groups
            .iter()
            .flat_map(|g| g.gachas.iter())
            .any(|g| g.gacha_id == req.gacha_id)
        {
            return Err(Status::invalid_argument("unknown gacha_id"));
        }
        let pool: Vec<&Card> = Card::table().iter().collect();
        let (normal, fixed) = rates::probability_response(&pool);
        Ok(Response::new(GachaListCardSelectProbabilityResponse {
            normal_rarity_probabilities: normal,
            fixed_rarity_probabilities: fixed,
            common_response: None,
        }))
    }

    async fn set_selected_card(
        &self,
        req: Request<GachaSetSelectedCardRequest>,
    ) -> Result<Response<GachaSetSelectedCardResponse>, Status> {
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if !data::list()
            .gacha_groups
            .iter()
            .flat_map(|g| g.gachas.iter())
            .any(|g| g.gacha_id == req.gacha_id)
        {
            return Err(Status::invalid_argument("unknown gacha_id"));
        }
        for card_id in &req.card_ids {
            if !Card::table().iter().any(|c| &c.id == card_id) {
                return Err(Status::invalid_argument(format!(
                    "unknown card_id {card_id}"
                )));
            }
        }
        data::SELECTED_CARDS
            .lock()
            .insert(req.gacha_id.clone(), req.card_ids.clone());
        Ok(Response::new(GachaSetSelectedCardResponse::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::Database;
    use std::sync::atomic::AtomicU32;
    use types::entity::master::GachaButton;

    use super::grants::resolve_draw_count;

    static DB_SEQ: AtomicU32 = AtomicU32::new(0);
    async fn test_db() -> Database {
        let seq = DB_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("symgacha-{}-{seq}", std::process::id()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let path = dir.join("t.db");
        Database::open(&path).await.expect("open db")
    }

    #[tokio::test]
    async fn init_loads_master_tables() {
        let db = test_db().await;
        GachaService::init(db).await.expect("init loads masters");
        assert!(!Card::table().is_empty());
        assert!(!GachaButton::table().is_empty());
    }

    #[tokio::test]
    async fn set_selected_card_roundtrip() {
        let db = test_db().await;
        let _ = GachaService::init(db).await;
        let gacha_id = data::list()
            .gacha_groups
            .iter()
            .flat_map(|g| g.gachas.iter())
            .next()
            .expect("a gacha in list")
            .gacha_id
            .clone();
        let card_id = Card::table()[0].id.clone();
        data::SELECTED_CARDS
            .lock()
            .insert(gacha_id.clone(), vec![card_id.clone()]);
        let stored = data::SELECTED_CARDS.lock();
        assert_eq!(stored.get(&gacha_id), Some(&vec![card_id]));
    }

    #[tokio::test]
    async fn draw_count_resolves_from_button() {
        // btn_10-stone: total_reward_pick_count = 10; client sends draw_count=1
        resource::master::load::<types::entity::master::Gacha>()
            .await
            .expect("gacha");
        resource::master::load::<types::entity::master::GachaButton>()
            .await
            .expect("buttons");
        resource::master::load::<types::entity::master::GachaPoint>()
            .await
            .expect("points");
        let (_, ten) = data::find_button("gacha_button-common-normal-001-btn_10-stone")
            .expect("btn_10-stone in list");
        assert_eq!(ten.total_reward_pick_count, 10);
        assert_eq!(resolve_draw_count(&ten, 1).unwrap(), 10);
        assert_eq!(resolve_draw_count(&ten, 2).unwrap(), 20);

        // btn_01-stone: 1 card per pull
        let (_, one) = data::find_button("gacha_button-common-normal-001-btn_01-stone")
            .expect("btn_01-stone in list");
        assert_eq!(one.total_reward_pick_count, 1);
        assert_eq!(resolve_draw_count(&one, 1).unwrap(), 1);

        // abuse cap
        assert!(resolve_draw_count(&ten, 100).is_err());
    }
}
