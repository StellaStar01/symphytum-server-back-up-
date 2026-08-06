use database::Database;
use tonic::{Request, Response, Status};
use types::Validate;
use types::rpc::api::character_history_list_response::History;
use types::rpc::api::character_history_server::CharacterHistory;
use types::rpc::api::{CharacterHistoryListRequest, CharacterHistoryListResponse};

#[derive(Clone)]
pub struct CharacterHistoryService {
    _db: Database,
}

impl CharacterHistoryService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl CharacterHistory for CharacterHistoryService {
    async fn list(
        &self,
        req: Request<CharacterHistoryListRequest>,
    ) -> Result<Response<CharacterHistoryListResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(CharacterHistoryListResponse {
            histories: Vec::<History>::new(),
            is_next: false,
            common_response: None,
        }))
    }
}
