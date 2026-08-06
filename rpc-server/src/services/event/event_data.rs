#![allow(dead_code)]

// Capture-derived reference only (rpc.api.Event_ListEventInfo_20260801_135516_668_RESP_0.bin);
// the server returns an empty event list.

use types::common;
use types::rpc::api::EventListEventInfoResponse;
use types::rpc::api::event_list_event_info_response;

pub fn response() -> EventListEventInfoResponse {
    EventListEventInfoResponse {
  event_infos: vec![
  event_list_event_info_response::EventInfo {
    event_id: "marathon_event-normal-001".into(),
    r#type: 1,
    name: "Point Rally Event".into(),
    start_time: 1785207600000,
    end_time: 1785927599000,
    exchange_end_time: 1786445999000,
    logo_asset_id: "img_marathon_event_normal_logo-001-01".into(),
    background_asset_id: "img_marathon_event_normal_bg-001-01".into(),
    is_new: true,
    is_noti: true,
    mission_group_id: "".into(),
    view_condition_group_id: "".into(),
    unlock_condition_group_id: "cd-marathon_event".into(),
    marathon_info: Some(
      event_list_event_info_response::event_info::MarathonInfo {
        exchange_booth_group_id: "exchange_booth_group-marathon_event-normal-001".into(),
        aggregation_start_time: 1785942000000,
        aggregated_ranking_reveal_start_time: 1785985200000,
        marathon_chapters: vec![
        event_list_event_info_response::event_info::marathon_info::MarathonChapter {
          id: "marathon_event-normal-001-chapter-01".into(),
          chapter_number: 1,
          character_id: "".into(),
          end_time: 1785927599000,
          score_name: "Event Pt".into(),
          score_icon_asset_id: "event-pt-01".into(),
          marathon_event_badge_item_id: "item-event_badge-marathon_event-normal-001-chapter-01".into(),
          mission_group_id: "mission_grp-marathon_event-normal-001-chapter-01".into(),
          marathon_score_rewards: vec![
          event_list_event_info_response::event_info::marathon_info::marathon_chapter::MarathonScoreReward {
            score: 30000,
            rewards: vec![
            common::Reward {
              resource_type: 24,
              resource_id: "story-marathon_event-normal-001-chapter-01-2".into(),
              quantity: 1,
            ..Default::default()
            },
            ],
          ..Default::default()
          },
          event_list_event_info_response::event_info::marathon_info::marathon_chapter::MarathonScoreReward {
            score: 60000,
            rewards: vec![
            common::Reward {
              resource_type: 24,
              resource_id: "story-marathon_event-normal-001-chapter-01-3".into(),
              quantity: 1,
            ..Default::default()
            },
            ],
          ..Default::default()
          },
          event_list_event_info_response::event_info::marathon_info::marathon_chapter::MarathonScoreReward {
            score: 100000,
            rewards: vec![
            common::Reward {
              resource_type: 24,
              resource_id: "story-marathon_event-normal-001-chapter-01-4".into(),
              quantity: 1,
            ..Default::default()
            },
            ],
          ..Default::default()
          },
          event_list_event_info_response::event_info::marathon_info::marathon_chapter::MarathonScoreReward {
            score: 150000,
            rewards: vec![
            common::Reward {
              resource_type: 24,
              resource_id: "story-marathon_event-normal-001-chapter-01-5".into(),
              quantity: 1,
            ..Default::default()
            },
            ],
          ..Default::default()
          },
          event_list_event_info_response::event_info::marathon_info::marathon_chapter::MarathonScoreReward {
            score: 200000,
            rewards: vec![
            common::Reward {
              resource_type: 24,
              resource_id: "story-marathon_event-normal-001-chapter-01-6".into(),
              quantity: 1,
            ..Default::default()
            },
            ],
          ..Default::default()
          },
          ],
          score: 0,
          event_story_chapter_id: "event_story_chapter-marathon_event-normal-001-chapter-01".into(),
          auto_play_story_id: "story-marathon_event-normal-001-chapter-01-1".into(),
          background_asset_id: "img_marathon_event_normal_top-001-01".into(),
          bgm_asset_id: "music_short_m0309".into(),
          ranking_reveal_start_time: 1785985200000,
        ..Default::default()
        },
        ],
        is_marathon_score_ranking_disable: false,
        tips_hint_type: 2100,
      ..Default::default()
      },    ),
  ..Default::default()
  },
  event_list_event_info_response::EventInfo {
    event_id: "event_mission-beginner_mission".into(),
    r#type: 2,
    name: "Beginner Mission".into(),
    start_time: 946652400000,
    end_time: 0,
    exchange_end_time: 0,
    logo_asset_id: "".into(),
    background_asset_id: "img_event_mission_bg_001".into(),
    is_new: true,
    is_noti: false,
    mission_group_id: "mission_grp-beginner_mission".into(),
    view_condition_group_id: "cd-event_mission-beginner_mission-close".into(),
    unlock_condition_group_id: "".into(),
    marathon_info: None,
  ..Default::default()
  },
  ],
..Default::default()
}
}
