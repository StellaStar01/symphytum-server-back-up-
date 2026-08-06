use database::Database;
use tonic::{Request, Response, Status};
use types::Validate;
use types::rpc::api::character_server::Character;
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::{
    CharacterReadParkRequest, CharacterReadParkResponse, CharacterReadRequest,
    CharacterReadResponse,
};

mod delta;

#[derive(Clone)]
pub struct CharacterService {
    db: Database,
}

impl CharacterService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Character for CharacterService {
    async fn read(
        &self,
        req: Request<CharacterReadRequest>,
    ) -> Result<Response<CharacterReadResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let now = database::unix_now() * 1000;
        self.db
            .mark_character_read(&uid, &req.character_id, now)
            .await
            .map_err(|e| Status::internal(format!("mark read: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CharacterReadResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_character_read(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }

    async fn read_park(
        &self,
        req: Request<CharacterReadParkRequest>,
    ) -> Result<Response<CharacterReadParkResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let now = database::unix_now() * 1000;
        self.db
            .mark_character_park_read(&uid, &req.character_id, now)
            .await
            .map_err(|e| Status::internal(format!("mark park read: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CharacterReadParkResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_character_read(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }
}
