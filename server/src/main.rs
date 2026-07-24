use resource::cert;
use tonic::{Request, Response, Status};
use tonic_middleware::MiddlewareLayer;
// use types::rpc::api::common::{Response as CommonResponse, UserData};
// use types::rpc::api::user_server::{User, UserServer};
// use types::rpc::api::{UserDeleteResponse, UserGetResponse};
use types::enums::GraphicsQualityType;
use types::rpc::api::system_get_system_info_response::GachaAssetInfo;
use types::rpc::api::system_get_system_info_response::MaintenanceInfo;
use types::rpc::api::system_get_system_info_response::ReviewInfo;
use types::rpc::api::system_server::{System, SystemServer};
use types::rpc::api::{SystemGetSystemInfoRequest, SystemGetSystemInfoResponse};

mod middleware;
use middleware::QualiCrypt;

mod whateverlol; // delete later, duh
use whateverlol::welcome;

#[derive(Default)]
pub struct MySystem {}

#[tonic::async_trait]
impl System for MySystem {
    async fn get_system_info(
        &self,
        _req: Request<SystemGetSystemInfoRequest>,
    ) -> Result<Response<SystemGetSystemInfoResponse>, Status> {
        println!("Hello");
        let mut shitass = SystemGetSystemInfoResponse::default();
        let hardcode = welcome();

        shitass.title_download_gacha_asset_infos = hardcode
            .title_download_gacha_asset_infos
            .iter()
            .map(|gc| GachaAssetInfo {
                promotion_pickup_card_ids: gc.promotion_pickup_card_ids.clone().into(),
                icon_asset_id: gc.icon_asset_id.clone(),
                promotion_movie_asset_id: gc.promotion_movie_asset_id.clone(),
                promotion_image_asset_id: gc.promotion_image_asset_id.clone(),
                bgm_asset_id: gc.bgm_asset_id.clone(),
                gacha_animation_asset_id: gc.gacha_animation_asset_id.clone(),
                gacha_point_icon_asset_id: gc.gacha_point_icon_asset_id.clone(),
            })
            .collect();

        shitass.device_workaround_types = hardcode
            .device_workaround_types
            .iter()
            .flatten()
            .map(|d| d.as_i64().unwrap() as i32)
            .collect();

        shitass.octo_distribution_host_name = hardcode.octo_distribution_host_name;
        shitass.octo_api_host_name = hardcode.octo_api_host_name;
        shitass.octo_asset_env_id = hardcode.octo_asset_env_id as i32;
        shitass.octo_distribution_version = hardcode.octo_distribution_version as i32;
        shitass.inquiry_api_url = hardcode.inquiry_api_url;

        shitass.review_info = Some(ReviewInfo {
            is_in_review: hardcode.review_info.is_in_review,
            api_host_in_review: hardcode.review_info.api_host_in_review.clone(),
            octo_distribution_host_name: hardcode.review_info.octo_distribution_host_name.clone(),
            octo_api_host_name: hardcode.review_info.octo_api_host_name.clone(),
            octo_asset_env_id: hardcode.review_info.octo_asset_env_id as i32,
            octo_distribution_version: hardcode.review_info.octo_distribution_version as i32,
        });

        shitass.maintenance_info = Some(MaintenanceInfo {
            is_in_maintenance: hardcode.maintenance_info.is_in_maintenance,
            is_prerelease: hardcode.maintenance_info.is_prerelease,
            start_time: hardcode
                .maintenance_info
                .start_time
                .clone()
                .parse()
                .unwrap(),
            end_time: hardcode.maintenance_info.end_time.clone().parse().unwrap(),
            description: hardcode.maintenance_info.description.clone(),
            character_asset_id: hardcode.maintenance_info.character_asset_id.clone(),
            character_color: hardcode.maintenance_info.character_color.clone(),
            is_skip_maintenance: hardcode.maintenance_info.is_skip_maintenance,
        });

        shitass.recommend_graphics_quality_type = GraphicsQualityType::High.into();

        Ok(Response::new(shitass))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3000".parse()?;

    println!("gRPC Server listening on {}", addr);

    tonic::transport::Server::builder()
        .tls_config(cert::get_tls_config())?
        .layer(MiddlewareLayer::new(QualiCrypt))
        .add_service(SystemServer::new(MySystem::default()))
        .serve(addr)
        .await?;

    Ok(())
}
