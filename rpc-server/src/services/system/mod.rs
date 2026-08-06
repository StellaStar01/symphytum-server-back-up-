use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::system_get_system_info_response::{
    GachaAssetInfo, MaintenanceInfo, ReviewInfo,
};
use types::rpc::api::system_server::System;
use types::rpc::api::{SystemGetSystemInfoRequest, SystemGetSystemInfoResponse};

mod data;

#[derive(Clone)]
pub struct SystemService {
    _db: Database,
}

impl SystemService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl System for SystemService {
    async fn get_system_info(
        &self,
        _req: Request<SystemGetSystemInfoRequest>,
    ) -> Result<Response<SystemGetSystemInfoResponse>, Status> {
        let pickup = data::pickup_cards();
        Ok(Response::new(SystemGetSystemInfoResponse {
            octo_distribution_host_name: "asset.game-hololive-dreams.com".into(),
            octo_api_host_name: "as.game-hololive-dreams.com/asset".into(),
            octo_asset_env_id: 5,
            octo_distribution_version: 1,
            inquiry_api_url: "https://inquiry-api-as.game-hololive-dreams.com".into(),
            review_info: Some(ReviewInfo {
                is_in_review: false,
                api_host_in_review: "https://us.review-game-hololive-dreams.com".into(),
                octo_distribution_host_name: "asset.review-game-hololive-dreams.com".into(),
                octo_api_host_name: "us.review-game-hololive-dreams.com/asset".into(),
                octo_asset_env_id: 4,
                octo_distribution_version: 1,
            }),
            maintenance_info: Some(MaintenanceInfo {
                is_in_maintenance: false,
                is_prerelease: false,
                start_time: 0,
                end_time: 0,
                description: String::new(),
                character_asset_id: String::new(),
                character_color: String::new(),
                is_skip_maintenance: false,
            }),
            title_download_gacha_asset_infos: vec![
                GachaAssetInfo {
                    icon_asset_id: "common-ticket-r5-001".into(),
                    promotion_image_asset_id: "img_gacha_top_gacha-common-ticket_r5-001".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    ..Default::default()
                },
                GachaAssetInfo {
                    icon_asset_id: "common-normal-001".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    gacha_point_icon_asset_id: "gacha-point-common-001".into(),
                    ..Default::default()
                },
                GachaAssetInfo {
                    icon_asset_id: "common-ticket-r4-001".into(),
                    promotion_image_asset_id: "img_gacha_top_gacha-common-ticket_r4-001".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    ..Default::default()
                },
                GachaAssetInfo {
                    icon_asset_id: "beginner-select-001".into(),
                    promotion_image_asset_id: "img_gacha_top_gacha-beginner-select-001".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    gacha_point_icon_asset_id: "gacha-point-common-001".into(),
                    ..Default::default()
                },
                GachaAssetInfo {
                    icon_asset_id: "fixed-beginner-001".into(),
                    promotion_image_asset_id: "img_gacha_top_gacha-fixed-beginner-001".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    ..Default::default()
                },
                GachaAssetInfo {
                    icon_asset_id: "pickup-normal-260728".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    gacha_point_icon_asset_id: "gacha-point-common-001".into(),
                    promotion_pickup_card_ids: pickup.clone(),
                    ..Default::default()
                },
                GachaAssetInfo {
                    icon_asset_id: "pickup-select-260728".into(),
                    gacha_animation_asset_id: "gacha_animation_movie-default-001".into(),
                    gacha_point_icon_asset_id: "gacha-point-common-001".into(),
                    promotion_pickup_card_ids: pickup,
                    ..Default::default()
                },
            ],
            recommend_graphics_quality_type: 0,
            device_workaround_types: vec![],
        }))
    }
}
