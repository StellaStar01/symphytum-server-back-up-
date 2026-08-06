use database::Database;
use rand::RngExt;
use resource::config::CONFIG;
use tonic::{Request, Response, Status};
use types::Validate;
use types::enums::CustomPaletteBackgroundResourceType;
use types::rpc::api::common::CustomPalette;
use types::rpc::api::common::CustomPaletteBackground;
use types::rpc::api::common::CustomPalettePart;
use types::rpc::api::common::Response as CommonResponse;
use types::rpc::api::common::StorageUrlInfo;
use types::rpc::api::common::UserData;
use types::rpc::api::common::UserProfileDetailInfo;
use types::rpc::api::profile_server::Profile;
use types::rpc::api::{
    ProfileBlockUserRequest, ProfileBlockUserResponse, ProfileComplainUserRequest,
    ProfileComplainUserResponse, ProfileDeleteCustomPaletteRequest,
    ProfileDeleteCustomPaletteResponse, ProfileEditCustomPaletteRequest,
    ProfileEditCustomPaletteResponse, ProfileGetCustomPaletteImageUploadUrlRequest,
    ProfileGetCustomPaletteImageUploadUrlResponse, ProfileGetUserProfileDetailRequest,
    ProfileGetUserProfileDetailResponse, ProfileListBlockedUserResponse,
    ProfileListCustomPaletteResponse, ProfileSetCustomPaletteRequest,
    ProfileSetCustomPaletteResponse, ProfileSetDefaultCustomPaletteResponse,
    ProfileSetEmblemRequest, ProfileSetEmblemResponse, ProfileSetFanMarkRequest,
    ProfileSetFanMarkResponse, ProfileSetMessageRequest, ProfileSetMessageResponse,
    ProfileSetNameRequest, ProfileSetNameResponse, ProfileSwitchPublishSettingRequest,
    ProfileSwitchPublishSettingResponse, ProfileUnblockUserRequest, ProfileUnblockUserResponse,
};

mod delta;

use crate::auth_token;

#[derive(Clone)]
pub struct ProfileService {
    db: Database,
}

impl ProfileService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }
}

#[tonic::async_trait]
impl Profile for ProfileService {
    async fn set_name(
        &self,
        req: Request<ProfileSetNameRequest>,
    ) -> Result<Response<ProfileSetNameResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        // the capture's timestamps are milliseconds
        let now = database::unix_now() * 1000;
        self.db
            .set_profile_name(&uid, &req.name, now)
            .await
            .map_err(|e| Status::internal(format!("set name: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(ProfileSetNameResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_profile(&updated)),
                ..Default::default()
            }),
        }))
    }

    async fn set_message(
        &self,
        req: Request<ProfileSetMessageRequest>,
    ) -> Result<Response<ProfileSetMessageResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let now = database::unix_now() * 1000;
        self.db
            .set_profile_message(&uid, &req.message, now)
            .await
            .map_err(|e| Status::internal(format!("set message: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(ProfileSetMessageResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_profile(&updated)),
                ..Default::default()
            }),
        }))
    }

    async fn set_fan_mark(
        &self,
        req: Request<ProfileSetFanMarkRequest>,
    ) -> Result<Response<ProfileSetFanMarkResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        // no buf.validate constraint on fan_mark_id; still require non-empty
        if req.fan_mark_id.is_empty() {
            return Err(Status::invalid_argument("fan_mark_id required"));
        }
        self.db
            .set_profile_fan_mark(&uid, &req.fan_mark_id)
            .await
            .map_err(|e| Status::internal(format!("set fan mark: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(ProfileSetFanMarkResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_profile(&updated)),
                ..Default::default()
            }),
        }))
    }

    async fn switch_publish_setting(
        &self,
        req: Request<ProfileSwitchPublishSettingRequest>,
    ) -> Result<Response<ProfileSwitchPublishSettingResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        req.into_inner();
        Ok(Response::new(ProfileSwitchPublishSettingResponse::default()))
    }

    async fn set_emblem(
        &self,
        req: Request<ProfileSetEmblemRequest>,
    ) -> Result<Response<ProfileSetEmblemResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let positions: Vec<(i32, String)> = req
            .emblem_positions
            .iter()
            .map(|p| (p.position, p.emblem_id.clone()))
            .collect();
        self.db
            .set_profile_emblem_positions(&uid, &positions)
            .await
            .map_err(|e| Status::internal(format!("set emblems: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        Ok(Response::new(ProfileSetEmblemResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(delta::after_profile(&updated)),
                ..Default::default()
            }),
        }))
    }

    async fn get_user_profile_detail(
        &self,
        req: Request<ProfileGetUserProfileDetailRequest>,
    ) -> Result<Response<ProfileGetUserProfileDetailResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if req.public_user_id != uid {
            // who cares
            return Err(Status::unimplemented(
                "Profile.get_user_profile_detail (other user)",
            ));
        }
        Ok(Response::new(ProfileGetUserProfileDetailResponse {
            user_profile_detail_info: Some(UserProfileDetailInfo {
                public_user_id: uid,
                ..Default::default()
            }),
            common_response: None,
        }))
    }

    async fn block_user(
        &self,
        req: Request<ProfileBlockUserRequest>,
    ) -> Result<Response<ProfileBlockUserResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(ProfileBlockUserResponse::default()))
    }

    async fn unblock_user(
        &self,
        req: Request<ProfileUnblockUserRequest>,
    ) -> Result<Response<ProfileUnblockUserResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(ProfileUnblockUserResponse::default()))
    }

    async fn list_blocked_user(
        &self,
        req: Request<()>,
    ) -> Result<Response<ProfileListBlockedUserResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        Ok(Response::new(ProfileListBlockedUserResponse::default()))
    }

    async fn complain_user(
        &self,
        req: Request<ProfileComplainUserRequest>,
    ) -> Result<Response<ProfileComplainUserResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(ProfileComplainUserResponse::default()))
    }

    async fn list_custom_palette(
        &self,
        req: Request<()>,
    ) -> Result<Response<ProfileListCustomPaletteResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let rows = self
            .db
            .custom_palettes(&uid)
            .await
            .map_err(|e| Status::internal(format!("palettes: {e}")))?;
        let mut custom_palettes = Vec::with_capacity(rows.len());
        for r in rows {
            if r.is_inactivated {
                continue;
            }
            let part_rows = self
                .db
                .custom_palette_parts(&uid, r.number)
                .await
                .map_err(|e| Status::internal(format!("palette parts: {e}")))?;
            custom_palettes.push(CustomPalette {
                number: r.number,
                custom_palette_preset_layout_group_id: String::new(),
                background: Some(CustomPaletteBackground {
                    resource_type: CustomPaletteBackgroundResourceType::Card as i32,
                    resource_id: r.background_card_id,
                    position_x_permil: 0,
                    position_y_permil: 0,
                    scale_permil: 1000,
                }),
                parts: part_rows
                    .into_iter()
                    .map(|p| CustomPalettePart {
                        resource_type: p.resource_type,
                        resource_id: p.resource_id,
                        position_x_permil: p.position_x_permil,
                        position_y_permil: p.position_y_permil,
                        scale_permil: p.scale_permil,
                        rotation_permil: p.rotation_permil,
                        layer: p.layer,
                    })
                    .collect(),
            });
        }
        Ok(Response::new(ProfileListCustomPaletteResponse {
            custom_palettes,
            common_response: None,
        }))
    }

    async fn get_custom_palette_image_upload_url(
        &self,
        req: Request<ProfileGetCustomPaletteImageUploadUrlRequest>,
    ) -> Result<Response<ProfileGetCustomPaletteImageUploadUrlResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        // the image is PUT to hattp; the same URL serves it back for display
        let mut rng = rand::rng();
        let token: String = (0..16)
            .map(|_| format!("{:02x}", rng.random_range(0..=255)))
            .collect();
        Ok(Response::new(
            ProfileGetCustomPaletteImageUploadUrlResponse {
                upload_token: token.clone(),
                storage_url_info: Some(StorageUrlInfo {
                    url: format!(
                        "http://{}:{}/palette_upload/{token}",
                        CONFIG.http_server.host, CONFIG.http_server.port
                    ),
                    content_type: "image/jpeg".into(),
                    method: "PUT".into(),
                    headers: vec![],
                }),
                common_response: None,
            },
        ))
    }

    async fn edit_custom_palette(
        &self,
        req: Request<ProfileEditCustomPaletteRequest>,
    ) -> Result<Response<ProfileEditCustomPaletteResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        if req.upload_token.is_empty() {
            return Err(Status::invalid_argument("upload_token required"));
        }
        // the client only renders palettes with number != 0; normalize 0 -> 1
        let number = req.number.max(1);
        // the palette image is served by hattp; the record survives restarts for user_data
        let image_url = format!(
            "http://{}:{}/palette/{}",
            CONFIG.http_server.host, CONFIG.http_server.port, number
        );
        // persist the background card the editor picked (type CARD only)
        let (background_card_id, background_potential) = match req.background.as_ref() {
            Some(bg) if bg.resource_type == CustomPaletteBackgroundResourceType::Card as i32 => {
                let potential = self
                    .db
                    .user_card(&uid, &bg.resource_id)
                    .await
                    .map_err(|e| Status::internal(format!("bg card: {e}")))?
                    .map(|c| c.potential_upgrade_count)
                    .unwrap_or(0);
                (bg.resource_id.clone(), potential)
            }
            _ => (String::new(), 0),
        };
        self.db
            .set_custom_palette(
                &uid,
                number,
                &image_url,
                &background_card_id,
                background_potential,
            )
            .await
            .map_err(|e| Status::internal(format!("save palette: {e}")))?;
        // the top screen renders the palette from background + parts, never the image
        self.db
            .set_custom_palette_parts(&uid, number, &req.parts)
            .await
            .map_err(|e| Status::internal(format!("save palette parts: {e}")))?;
        // the client shows the palette whose number == user_profile.custom_palette_number
        self.db
            .set_profile_custom_palette(&uid, number)
            .await
            .map_err(|e| Status::internal(format!("activate palette: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        let mut d = UserData::default();
        d.user_custom_palette_list = updated.user_custom_palette_list.clone();
        d.user_profile = updated.user_profile.clone();
        Ok(Response::new(ProfileEditCustomPaletteResponse {
            number,
            common_response: Some(CommonResponse {
                updated_data: Some(d),
                ..Default::default()
            }),
        }))
    }

    async fn set_custom_palette(
        &self,
        req: Request<ProfileSetCustomPaletteRequest>,
    ) -> Result<Response<ProfileSetCustomPaletteResponse>, Status> {
        let uid = auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.db
            .set_profile_custom_palette(&uid, req.number)
            .await
            .map_err(|e| Status::internal(format!("activate palette: {e}")))?;
        let updated = self
            .db
            .user_data(&uid)
            .await
            .map_err(|e| Status::internal(format!("snapshot: {e}")))?;
        let mut d = UserData::default();
        d.user_profile = updated.user_profile.clone();
        Ok(Response::new(ProfileSetCustomPaletteResponse {
            common_response: Some(CommonResponse {
                updated_data: Some(d),
                ..Default::default()
            }),
        }))
    }

    async fn delete_custom_palette(
        &self,
        req: Request<ProfileDeleteCustomPaletteRequest>,
    ) -> Result<Response<ProfileDeleteCustomPaletteResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(ProfileDeleteCustomPaletteResponse::default()))
    }

    async fn set_default_custom_palette(
        &self,
        req: Request<()>,
    ) -> Result<Response<ProfileSetDefaultCustomPaletteResponse>, Status> {
        let _ = auth_token::uid(&req)?;
        Ok(Response::new(
            ProfileSetDefaultCustomPaletteResponse::default(),
        ))
    }
}
