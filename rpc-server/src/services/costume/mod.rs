use database::Database;
use tonic::{Request, Response, Status};
use types::Validate;
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::costume_server::Costume;
use types::rpc::api::{
    CostumeReadCostumeRequest, CostumeReadCostumeResponse,
    CostumeReadSdCostumeHairAccessoryRequest, CostumeReadSdCostumeHairAccessoryResponse,
    CostumeReadSdCostumeRequest, CostumeReadSdCostumeResponse, CostumeSetCostumeRequest,
    CostumeSetCostumeResponse, CostumeSetSdCostumeRequest, CostumeSetSdCostumeResponse,
};

mod delta;

#[derive(Clone)]
pub struct CostumeService {
    db: Database,
}

impl CostumeService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Costume for CostumeService {
    async fn set_costume(
        &self,
        req: Request<CostumeSetCostumeRequest>,
    ) -> Result<Response<CostumeSetCostumeResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if !self
            .db
            .owns_costume(&uid, &req.costume_id)
            .await
            .map_err(|e| Status::internal(format!("costume: {e}")))?
        {
            return Err(Status::invalid_argument("costume not owned"));
        }
        self.db
            .set_character_costume(&uid, &req.character_id, &req.costume_id)
            .await
            .map_err(|e| Status::internal(format!("set costume: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CostumeSetCostumeResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_character(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }

    async fn set_sd_costume(
        &self,
        req: Request<CostumeSetSdCostumeRequest>,
    ) -> Result<Response<CostumeSetSdCostumeResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if !self
            .db
            .owns_sd_costume(&uid, &req.sd_costume_id)
            .await
            .map_err(|e| Status::internal(format!("sd costume: {e}")))?
        {
            return Err(Status::invalid_argument("sd costume not owned"));
        }
        self.db
            .set_character_sd_costume(
                &uid,
                &req.character_id,
                &req.sd_costume_id,
                &req.sd_costume_hair_accessory_id,
            )
            .await
            .map_err(|e| Status::internal(format!("set sd costume: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CostumeSetSdCostumeResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_character(&updated, &req.character_id)),
                ..Default::default()
            }),
        }))
    }

    async fn read_costume(
        &self,
        req: Request<CostumeReadCostumeRequest>,
    ) -> Result<Response<CostumeReadCostumeResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let ids = self
            .db
            .character_costume_ids(&uid, &req.character_id)
            .await
            .map_err(|e| Status::internal(format!("character: {e}")))?
            .ok_or_else(|| Status::invalid_argument("character not owned"))?;
        let now = database::unix_now() * 1000;
        if !ids.0.is_empty() {
            self.db
                .mark_costume_read(&uid, &ids.0, now)
                .await
                .map_err(|e| Status::internal(format!("mark read: {e}")))?;
        }
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CostumeReadCostumeResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_costume_read(&updated, &ids.0, &ids.1, &ids.2)),
                ..Default::default()
            }),
        }))
    }

    async fn read_sd_costume(
        &self,
        req: Request<CostumeReadSdCostumeRequest>,
    ) -> Result<Response<CostumeReadSdCostumeResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let ids = self
            .db
            .character_costume_ids(&uid, &req.character_id)
            .await
            .map_err(|e| Status::internal(format!("character: {e}")))?
            .ok_or_else(|| Status::invalid_argument("character not owned"))?;
        let now = database::unix_now() * 1000;
        if !ids.1.is_empty() {
            self.db
                .mark_sd_costume_read(&uid, &ids.1, now)
                .await
                .map_err(|e| Status::internal(format!("mark read: {e}")))?;
        }
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CostumeReadSdCostumeResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_costume_read(&updated, &ids.0, &ids.1, &ids.2)),
                ..Default::default()
            }),
        }))
    }

    async fn read_sd_costume_hair_accessory(
        &self,
        req: Request<CostumeReadSdCostumeHairAccessoryRequest>,
    ) -> Result<Response<CostumeReadSdCostumeHairAccessoryResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let ids = self
            .db
            .character_costume_ids(&uid, &req.character_id)
            .await
            .map_err(|e| Status::internal(format!("character: {e}")))?
            .ok_or_else(|| Status::invalid_argument("character not owned"))?;
        let now = database::unix_now() * 1000;
        if !ids.2.is_empty() {
            self.db
                .mark_sd_costume_hair_accessory_read(&uid, &ids.2, now)
                .await
                .map_err(|e| Status::internal(format!("mark read: {e}")))?;
        }
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(CostumeReadSdCostumeHairAccessoryResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_costume_read(&updated, &ids.0, &ids.1, &ids.2)),
                ..Default::default()
            }),
        }))
    }
}
