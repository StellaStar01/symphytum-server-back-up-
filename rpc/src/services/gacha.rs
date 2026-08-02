use tonic::{Request, Response, Status};

use types::rpc::api::gacha_server::Gacha;
use types::rpc::api::{
    GachaDrawCardSelectRequest, GachaDrawCardSelectResponse, GachaDrawNormalRequest,
    GachaDrawNormalResponse, GachaListCardSelectProbabilityRequest,
    GachaListCardSelectProbabilityResponse, GachaListHistoryResponse,
    GachaListNormalProbabilityRequest, GachaListNormalProbabilityResponse, GachaListResponse,
    GachaReadRequest, GachaReadResponse, GachaSetSelectedCardRequest, GachaSetSelectedCardResponse,
};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct GachaService {}

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
        Err(Status::unimplemented("Gacha.list_history"))
    }

    async fn read(
        &self,
        _req: Request<GachaReadRequest>,
    ) -> Result<Response<GachaReadResponse>, Status> {
        Err(Status::unimplemented("Gacha.read"))
    }

    async fn draw_normal(
        &self,
        _req: Request<GachaDrawNormalRequest>,
    ) -> Result<Response<GachaDrawNormalResponse>, Status> {
        Err(Status::unimplemented("Gacha.draw_normal"))
    }

    async fn list_normal_probability(
        &self,
        _req: Request<GachaListNormalProbabilityRequest>,
    ) -> Result<Response<GachaListNormalProbabilityResponse>, Status> {
        Err(Status::unimplemented("Gacha.list_normal_probability"))
    }

    async fn draw_card_select(
        &self,
        _req: Request<GachaDrawCardSelectRequest>,
    ) -> Result<Response<GachaDrawCardSelectResponse>, Status> {
        Err(Status::unimplemented("Gacha.draw_card_select"))
    }

    async fn list_card_select_probability(
        &self,
        _req: Request<GachaListCardSelectProbabilityRequest>,
    ) -> Result<Response<GachaListCardSelectProbabilityResponse>, Status> {
        Err(Status::unimplemented("Gacha.list_card_select_probability"))
    }

    async fn set_selected_card(
        &self,
        _req: Request<GachaSetSelectedCardRequest>,
    ) -> Result<Response<GachaSetSelectedCardResponse>, Status> {
        Err(Status::unimplemented("Gacha.set_selected_card"))
    }
}
