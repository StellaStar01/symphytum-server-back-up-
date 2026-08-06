// Derived from capture rpc.api.Park_Enter_20260806_150942_358_RESP_0.bin.
// No master tables back these banners; regenerate from a new capture.

use types::rpc::api::ParkEnterResponse;
use types::rpc::api::park_enter_response::{HomeBanner, ParkBanner, ParkBannerFacility};

pub fn response() -> ParkEnterResponse {
    ParkEnterResponse {
        home_banners: vec![
            HomeBanner {
                home_banner_id: "home_banner-event_mission-training_support-260816".into(),
                asset_id: "img_banner_full_event-mission-training-support-260816".into(),
                transition_type: 1,
                transition_id: "event_mission-training_support-260816".into(),
                end_time: 1787882399000,
                priority: 260806010,
                condition_group_id: "".into(),
                title: "Event".into(),
                start_time: 1785981600000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-gift-sales_ranking_1st-260729".into(),
                asset_id: "img_banner_full_gift-sales-ranking-1st-gacha-260728".into(),
                transition_type: 2,
                transition_id: "notice-news-gift-sales_ranking_1st-260729".into(),
                end_time: 1786067999000,
                priority: 260729010,
                condition_group_id: "".into(),
                title: "".into(),
                start_time: 1785308400000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-gacha-pickup-260728".into(),
                asset_id: "img_banner_full_gacha-pickup-260728".into(),
                transition_type: 1800,
                transition_id: "gacha-pickup-select-260728".into(),
                end_time: 1786067999000,
                priority: 260728020,
                condition_group_id: "".into(),
                title: "Gacha".into(),
                start_time: 1785200400000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-gift-sales_ranking_1st-260723".into(),
                asset_id: "img_banner_full_gift-sales-ranking-1st-gacha-260723".into(),
                transition_type: 2,
                transition_id: "notice-news-gift-sales_ranking_1st-260723".into(),
                end_time: 1786067999000,
                priority: 260726020,
                condition_group_id: "".into(),
                title: "".into(),
                start_time: 1785060000000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-music_campaign-release".into(),
                asset_id: "img_banner_full_music-campaign-release".into(),
                transition_type: 2,
                transition_id: "notice-news-info_campaign-release".into(),
                end_time: 1786132799000,
                priority: 260723040,
                condition_group_id: "".into(),
                title: "".into(),
                start_time: 946652400000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-gacha-beginner-select-001".into(),
                asset_id: "img_banner_full_gacha-beginner-select-001".into(),
                transition_type: 1800,
                transition_id: "gacha-beginner-select-001".into(),
                end_time: 0,
                priority: 260723030,
                condition_group_id: "cd-gacha-beginner-select-001-banner-end".into(),
                title: "Gacha".into(),
                start_time: 946652400000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-gacha-fixed-beginner-001".into(),
                asset_id: "img_banner_full_gacha-fixed-beginner-001".into(),
                transition_type: 1800,
                transition_id: "gacha-fixed-beginner-001".into(),
                end_time: 0,
                priority: 260723030,
                condition_group_id: "cd-gacha-fixed-beginner-001-banner-end".into(),
                title: "Gacha".into(),
                start_time: 946652400000,
                external_url: "".into(),
                ..Default::default()
            },
            HomeBanner {
                home_banner_id: "home_banner-event_mission-beginner_mission".into(),
                asset_id: "img_banner_full_event-mission-beginner-mission".into(),
                transition_type: 1,
                transition_id: "event_mission-beginner_mission".into(),
                end_time: 0,
                priority: 260723010,
                condition_group_id: "cd-event_mission-beginner_mission-close".into(),
                title: "Event".into(),
                start_time: 946652400000,
                external_url: "".into(),
                ..Default::default()
            },
        ],
        park_banners: vec![
            ParkBanner {
                park_banner_id: "park_banner-event_mission-training_support-260816".into(),
                r#type: 1,
                asset_id: "img_banner_full_event-mission-training-support-260816".into(),
                transition_type: 1,
                transition_id: "event_mission-training_support-260816".into(),
                end_time: 1787882399000,
                priority: 260806010,
                condition_group_id: "".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-gift-sales_ranking_1st-260729".into(),
                r#type: 1,
                asset_id: "img_banner_full_gift-sales-ranking-1st-gacha-260728".into(),
                transition_type: 2,
                transition_id: "notice-news-gift-sales_ranking_1st-260729".into(),
                end_time: 1786067999000,
                priority: 260729010,
                condition_group_id: "".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-pr-jr_central".into(),
                r#type: 1,
                asset_id: "img_banner_full_sub-jr-central".into(),
                transition_type: 4,
                transition_id: "".into(),
                end_time: 1798055999000,
                priority: 260728020,
                condition_group_id: "cd-banner-birthday-jack-nega".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard2".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard3".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard2".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard3".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-gacha-pickup-260728".into(),
                r#type: 1,
                asset_id: "img_banner_full_gacha-pickup-260728".into(),
                transition_type: 1800,
                transition_id: "gacha-pickup-select-260728".into(),
                end_time: 1786067999000,
                priority: 260728020,
                condition_group_id: "".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-pr-official_shop".into(),
                r#type: 1,
                asset_id: "img_banner_full_sub-official-shop".into(),
                transition_type: 0,
                transition_id: "".into(),
                end_time: 1786931999000,
                priority: 260728010,
                condition_group_id: "cd-banner-birthday-jack-nega".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard2".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard3".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard2".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard3".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-gift-sales_ranking_1st-260723".into(),
                r#type: 1,
                asset_id: "img_banner_full_gift-sales-ranking-1st-gacha-260723".into(),
                transition_type: 2,
                transition_id: "notice-news-gift-sales_ranking_1st-260723".into(),
                end_time: 1786067999000,
                priority: 260726020,
                condition_group_id: "".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-pr-nepox".into(),
                r#type: 1,
                asset_id: "img_banner_full_sub-nepox".into(),
                transition_type: 4,
                transition_id: "".into(),
                end_time: 1790539199000,
                priority: 260723040,
                condition_group_id: "cd-banner-birthday-jack-nega".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard2".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard3".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard2".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard3".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-music_campaign-release".into(),
                r#type: 1,
                asset_id: "img_banner_full_music-campaign-release".into(),
                transition_type: 2,
                transition_id: "notice-news-info_campaign-release".into(),
                end_time: 1786132799000,
                priority: 260723040,
                condition_group_id: "".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-pr-kitunezoku_mikkai".into(),
                r#type: 1,
                asset_id: "img_banner_full_sub-kitunezoku-mikkai".into(),
                transition_type: 4,
                transition_id: "".into(),
                end_time: 1786067999000,
                priority: 260723030,
                condition_group_id: "cd-banner-birthday-jack-nega".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard2".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard3".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard2".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard3".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-gacha-beginner-select-001".into(),
                r#type: 1,
                asset_id: "img_banner_full_gacha-beginner-select-001".into(),
                transition_type: 1800,
                transition_id: "gacha-beginner-select-001".into(),
                end_time: 0,
                priority: 260723030,
                condition_group_id: "cd-gacha-beginner-select-001-banner-end".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-pr-holoplus".into(),
                r#type: 1,
                asset_id: "img_banner_full_sub-holoplus".into(),
                transition_type: 4,
                transition_id: "".into(),
                end_time: 1786067999000,
                priority: 260723020,
                condition_group_id: "cd-banner-birthday-jack-nega".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard2".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard3".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard2".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard3".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-pr-official_game_website".into(),
                r#type: 1,
                asset_id: "img_banner_full_sub-official-game-website".into(),
                transition_type: 4,
                transition_id: "".into(),
                end_time: 0,
                priority: 260723010,
                condition_group_id: "cd-banner-birthday-jack-nega".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard2".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard3".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard2".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard3".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ParkBanner {
                park_banner_id: "park_banner-event_mission-beginner_mission".into(),
                r#type: 1,
                asset_id: "img_banner_full_event-mission-beginner-mission".into(),
                transition_type: 1,
                transition_id: "event_mission-beginner_mission".into(),
                end_time: 0,
                priority: 260723010,
                condition_group_id: "cd-event_mission-beginner_mission-close".into(),
                park_banner_facilities: vec![
                    ParkBannerFacility {
                        facility_id: "facility-a01-signboard1".into(),
                        view_condition_group_id: "".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard1".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                    ParkBannerFacility {
                        facility_id: "facility-a02-signboard4".into(),
                        view_condition_group_id: "cd-quest_clear-quest-main-w1-02-07-1-1".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        ],
        startup_notifications: vec![],
        music_startup_notifications: vec![],
        cheat_feature_restriction_types: vec![],
        ..Default::default()
    }
}
