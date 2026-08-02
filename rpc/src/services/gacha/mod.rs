use tonic::{Request, Response, Status};

use resource::master::{self, MasterTable};
use types::entity::master::{Card, GachaButton};
use types::rpc::api::common::RewardResult;
use types::rpc::api::gacha_server::Gacha;
use types::rpc::api::{
    GachaDrawCardSelectRequest, GachaDrawCardSelectResponse, GachaDrawNormalRequest,
    GachaDrawNormalResponse, GachaDrawResponseGachaButton, GachaListCardSelectProbabilityRequest,
    GachaListCardSelectProbabilityResponse, GachaListHistoryResponse,
    GachaListNormalProbabilityRequest, GachaListNormalProbabilityResponse, GachaListResponse,
    GachaReadRequest, GachaReadResponse, GachaSetSelectedCardRequest, GachaSetSelectedCardResponse,
    GachaListResponseGachaButton
};

use crate::services::replay;
use crate::sniffs;

mod data;
mod draw;
mod rates;

#[derive(Default)]
pub struct GachaService {}

impl GachaService {
    pub async fn init() -> Result<Self, String> {
        master::load::<Card>().await?;
        master::load::<GachaButton>().await?;
        Ok(Self::default())
    }
}

fn bonus_reward_results(
    button: &GachaListResponseGachaButton,
) -> Vec<RewardResult> {
    button
        .bonus_rewards
        .iter()
        .map(|r| RewardResult {
            resource_type: r.resource_type,
            resource_id: r.resource_id.clone(),
            quantity: r.quantity,
            before_quantity: 0,
            after_quantity: r.quantity,
            is_new: false,
            is_gift: false,
        })
        .collect()
}

// Cards per request: the client always sends draw_count=1 even for the 10-pull
// button; the per-pull card count lives on the button (total_reward_pick_count)
fn resolve_draw_count(
    button: &GachaListResponseGachaButton,
    draw_count: i32,
) -> Result<usize, Status> {
    let per_pull = button.total_reward_pick_count.max(1) as usize;
    let count = (draw_count.max(1) as usize).saturating_mul(per_pull);
    if count > 100 {
        return Err(Status::invalid_argument("too many draws"));
    }
    Ok(count)
}

#[tonic::async_trait]
impl Gacha for GachaService {
    async fn list(&self, _req: Request<()>) -> Result<Response<GachaListResponse>, Status> {
        Ok(Response::new(replay!(
            GachaListResponse,
            sniffs::GACHA_LIST_RESP
        )))
    }
    async fn list_history(
        &self,
        _req: Request<()>,
    ) -> Result<Response<GachaListHistoryResponse>, Status> {
        Ok(Response::new(GachaListHistoryResponse::default()))
    }

    async fn read(
        &self,
        _req: Request<GachaReadRequest>,
    ) -> Result<Response<GachaReadResponse>, Status> {
        // marks the gacha notice read; no state to update
        Ok(Response::new(GachaReadResponse::default()))
    }

    async fn draw_normal(
        &self,
        req: Request<GachaDrawNormalRequest>,
    ) -> Result<Response<GachaDrawNormalResponse>, Status> {
        let req = req.into_inner();
        let Some((_gacha, button)) = data::find_button(&req.gacha_button_id) else {
            return Err(Status::invalid_argument("unknown gacha_button_id"));
        };
        let count = resolve_draw_count(button, req.draw_count)?;
        let pool: Vec<&Card> = Card::table().iter().collect();
        let mut rng = rand::rng();
        let results = draw::draw(
            &mut rng,
            &pool,
            count,
            button.fixed_reward_pick_count > 0,
            None,
        );
        Ok(Response::new(GachaDrawNormalResponse {
            card_results: results,
            bonus_reward_results: bonus_reward_results(button),
            gacha_button: Some(GachaDrawResponseGachaButton {
                gacha_button_id: req.gacha_button_id.clone(),
                is_disabled: false,
                drawn_count: count as i32,
            }),
            common_response: None,
        }))
    }

    async fn list_normal_probability(
        &self,
        req: Request<GachaListNormalProbabilityRequest>,
    ) -> Result<Response<GachaListNormalProbabilityResponse>, Status> {
        let req = req.into_inner();
        if !data::LIST
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
        let req = req.into_inner();
        let Some((_gacha, button)) = data::find_button(&req.gacha_button_id) else {
            return Err(Status::invalid_argument("unknown gacha_button_id"));
        };
        let count = resolve_draw_count(button, req.draw_count)?;
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
        let mut rng = rand::rng();
        let results = draw::draw(
            &mut rng,
            &pool,
            count,
            button.fixed_reward_pick_count > 0,
            (!selection.is_empty()).then_some(&selection[..]),
        );
        Ok(Response::new(GachaDrawCardSelectResponse {
            card_results: results,
            bonus_reward_results: bonus_reward_results(button),
            gacha_button: Some(GachaDrawResponseGachaButton {
                gacha_button_id: req.gacha_button_id.clone(),
                is_disabled: false,
                drawn_count: count as i32,
            }),
            common_response: None,
        }))
    }

    async fn list_card_select_probability(
        &self,
        req: Request<GachaListCardSelectProbabilityRequest>,
    ) -> Result<Response<GachaListCardSelectProbabilityResponse>, Status> {
        let req = req.into_inner();
        if !data::LIST
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
        if !data::LIST
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
    use types::entity::master::GachaButton;

    #[tokio::test]
    async fn init_loads_master_tables() {
        GachaService::init().await.expect("init loads masters");
        assert!(!Card::table().is_empty());
        assert!(!GachaButton::table().is_empty());
    }

    #[tokio::test]
    async fn set_selected_card_roundtrip() {
        let _ = GachaService::init().await;
        let gacha_id = data::LIST
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

    #[test]
    fn draw_count_resolves_from_button() {
        // btn_10-stone: total_reward_pick_count = 10; client sends draw_count=1
        let (_, ten) = data::find_button("gacha_button-common-normal-001-btn_10-stone")
            .expect("btn_10-stone in list");
        assert_eq!(ten.total_reward_pick_count, 10);
        assert_eq!(resolve_draw_count(ten, 1).unwrap(), 10);
        assert_eq!(resolve_draw_count(ten, 2).unwrap(), 20);

        // btn_01-stone: 1 card per pull
        let (_, one) = data::find_button("gacha_button-common-normal-001-btn_01-stone")
            .expect("btn_01-stone in list");
        assert_eq!(one.total_reward_pick_count, 1);
        assert_eq!(resolve_draw_count(one, 1).unwrap(), 1);

        // abuse cap
        assert!(resolve_draw_count(ten, 100).is_err());
    }
}
