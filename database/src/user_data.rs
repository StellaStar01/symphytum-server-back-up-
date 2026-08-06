use sqlx::{Error, Sqlite};
use types::common::EmblemPosition;
use types::entity::master::{Card, ExchangeBoothFixedItem, ShopChargeItemProduct};
use types::entity::transaction::user_music_character_highest_score::HighestScoreInfo;
use types::entity::transaction::*;
use types::rpc::api::common::UserData;

use crate::models::*;

use resource::master::MasterTable;

use super::Database;

macro_rules! load_rows {
    ($self:expr, $uid:expr, $t:ty, $sql:literal) => {
        sqlx::query_as::<_, $t>($sql)
            .bind($uid)
            .fetch_all($self.pool())
            .await?
    };
}

impl Database {
    pub async fn build_user_data(&self, uid: &str) -> Result<UserData, Error> {
        let mut data = UserData::default();

        // --- 1:1 rows ---
        if let Some(r) = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE account_id = ?")
            .bind(uid)
            .fetch_optional(self.pool())
            .await?
        {
            data.user = Some(User {
                public_user_id: r.account_id.clone(),
                region: r.region,
                country_code: r.country_code,
                birth_year_for_payment: r.birth_year_for_payment,
                birth_month_for_payment: r.birth_month_for_payment,
                active_user_type: r.active_user_type,
                tutorial_cleared_time: r.tutorial_cleared_time,
                deleted_time: r.deleted_time,
            });
        }

        if let Some(r) =
            sqlx::query_as::<_, UserProfileRow>("SELECT * FROM user_profiles WHERE account_id = ?")
                .bind(uid)
                .fetch_optional(self.pool())
                .await?
        {
            let deck_cards: Vec<ProfileDeckCardRow> = load_rows!(
                self,
                uid,
                ProfileDeckCardRow,
                "SELECT * FROM user_profile_highest_deck_cards WHERE account_id = ? ORDER BY position"
            );
            let emblems: Vec<EmblemPositionRow> = load_rows!(
                self,
                uid,
                EmblemPositionRow,
                "SELECT * FROM user_profile_emblem_positions WHERE account_id = ? ORDER BY position"
            );
            data.user_profile = Some(UserProfile {
                name: r.name,
                message: r.message,
                park_character_id: r.park_character_id,
                fan_mark_id: r.fan_mark_id,
                is_public_user_id_publish: r.is_public_user_id_publish,
                is_basic_info_publish: r.is_basic_info_publish,
                is_character_rank_publish: r.is_character_rank_publish,
                is_live_result_publish: r.is_live_result_publish,
                is_mini_game_result_publish: r.is_mini_game_result_publish,
                is_user_info_publish_in_multi_game: r.is_user_info_publish_in_multi_game,
                custom_palette_number: r.custom_palette_number,
                exp: r.exp,
                highest_live_deck_evaluation_value: r.highest_live_deck_evaluation_value,
                highest_live_deck_evaluation_character_id: r
                    .highest_live_deck_evaluation_character_id,
                highest_live_deck_evaluation_costume_id: r.highest_live_deck_evaluation_costume_id,
                highest_live_deck_evaluation_card_ids: deck_cards
                    .iter()
                    .map(|d| d.card_id.clone())
                    .collect(),
                highest_live_deck_evaluation_deck_card_levels: deck_cards
                    .iter()
                    .map(|d| d.level)
                    .collect(),
                highest_live_deck_evaluation_deck_card_potential_upgrade_counts: deck_cards
                    .iter()
                    .map(|d| d.potential_upgrade_count)
                    .collect(),
                name_last_updated_time: r.name_last_updated_time,
                is_official: r.is_official,
                login_status_last_updated_time: r.login_status_last_updated_time,
                multi_game_unpublished_user_name: r.multi_game_unpublished_user_name,
                message_last_updated_time: r.message_last_updated_time,
                emblem_positions: emblems
                    .into_iter()
                    .map(|e| EmblemPosition {
                        position: e.position,
                        emblem_id: e.emblem_id,
                    })
                    .collect(),
            });
        }

        if let Some(r) =
            sqlx::query_as::<_, UserTimeRow>("SELECT * FROM user_times WHERE account_id = ?")
                .bind(uid)
                .fetch_optional(self.pool())
                .await?
        {
            data.user_time = Some(UserTime {
                game_start_time: r.game_start_time,
                last_login_time: r.last_login_time,
                force_relogin_flag_set_time: r.force_relogin_flag_set_time,
                last_login_os: r.last_login_os,
                comebacked_time: r.comebacked_time,
            });
        }

        if let Some(r) =
            sqlx::query_as::<_, UserBalanceRow>("SELECT * FROM user_balances WHERE account_id = ?")
                .bind(uid)
                .fetch_optional(self.pool())
                .await?
        {
            data.user_balance_free = Some(UserBalanceFree {
                free_quantity: r.free_quantity,
            });
            data.user_balance_paid = Some(UserBalancePaid {
                paid_quantity: r.paid_quantity,
            });
        }

        if let Some(r) = sqlx::query_as::<_, UserAppReviewRow>(
            "SELECT * FROM user_app_reviews WHERE account_id = ?",
        )
        .bind(uid)
        .fetch_optional(self.pool())
        .await?
        {
            data.user_app_review = Some(UserAppReview {
                is_displayed: r.is_displayed,
            });
        }

        // --- lists ---
        data.user_count_list = load_rows!(
            self,
            uid,
            UserCountRow,
            "SELECT * FROM user_counts WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserCount {
            count_type: r.count_type,
            total_count: r.total_count,
            daily_count: r.daily_count,
            last_updated_time: r.last_updated_time,
        })
        .collect();

        data.user_item_list = load_rows!(
            self,
            uid,
            UserItemRow,
            "SELECT * FROM user_items WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserItem {
            item_id: r.item_id,
            expired_time: r.expired_time,
            quantity: r.quantity,
            last_acquired_time: r.last_acquired_time,
        })
        .collect();

        data.user_item_owned_list = load_rows!(
            self,
            uid,
            UserItemOwnedRow,
            "SELECT * FROM user_item_owned WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserItemOwned { item_id: r.item_id })
        .collect();

        data.user_card_list = load_rows!(
            self,
            uid,
            UserCardRow,
            "SELECT * FROM user_cards WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserCard {
            card_id: r.card_id,
            exp: r.exp,
            level_limit_break_count: r.level_limit_break_count,
            potential_upgrade_count: r.potential_upgrade_count,
            potential_upgrade_point_quantity: r.potential_upgrade_point_quantity,
            acquired_time: r.acquired_time,
        })
        .collect();

        data.user_character_list = load_rows!(
            self,
            uid,
            UserCharacterRow,
            "SELECT * FROM user_characters WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserCharacter {
            character_id: r.character_id,
            costume_id: r.costume_id,
            sd_costume_id: r.sd_costume_id,
            sd_costume_hair_accessory_id: r.sd_costume_hair_accessory_id,
            exp: r.exp,
            highest_live_deck_evaluation_value: r.highest_live_deck_evaluation_value,
            acquired_time: r.acquired_time,
            last_reward_received_level: r.last_reward_received_level,
            read_time: r.read_time,
            park_read_time: r.park_read_time,
        })
        .collect();

        // skill trees: group child rows per character
        {
            let released: Vec<SkillTreeEntryRow> = load_rows!(
                self,
                uid,
                SkillTreeEntryRow,
                "SELECT * FROM user_character_skill_tree_released WHERE account_id = ?"
            );
            let connected: Vec<SkillTreeEntryRow> = load_rows!(
                self,
                uid,
                SkillTreeEntryRow,
                "SELECT * FROM user_character_skill_tree_connected WHERE account_id = ?"
            );
            let connected_cards: Vec<SkillTreeCardRow> = load_rows!(
                self,
                uid,
                SkillTreeCardRow,
                "SELECT * FROM user_character_skill_tree_connected_cards WHERE account_id = ?"
            );
            let mut chars: Vec<String> = released.iter().map(|r| r.character_id.clone()).collect();
            chars.extend(connected.iter().map(|r| r.character_id.clone()));
            chars.extend(connected_cards.iter().map(|r| r.character_id.clone()));
            chars.sort();
            chars.dedup();
            for ch in chars {
                let mut groups: Vec<String> = connected
                    .iter()
                    .filter(|r| r.character_id == ch)
                    .map(|r| r.node_group_id.clone())
                    .collect();
                // the client pairs connected skill-tree cards with groups by index; the lists must stay aligned
                let mut cards: Vec<String> = connected_cards
                    .iter()
                    .filter(|r| r.character_id == ch)
                    .map(|r| r.card_id.clone())
                    .collect();
                if cards.len() != groups.len() && !groups.is_empty() {
                    // only cards with a connect effect can be connected; connected nodes dereference that effect id
                    let pool: Vec<String> = match Card::try_table() {
                        Some(rows) => rows
                            .iter()
                            .filter(|c| {
                                c.character_id == ch && !c.skill_tree_connect_effect_id.is_empty()
                            })
                            .map(|c| c.id.clone())
                            .collect(),
                        None => vec![],
                    };
                    if pool.is_empty() {
                        // no connectable card exists for this character; drop the fake state
                        groups.clear();
                        cards.clear();
                    } else {
                        cards = (0..groups.len())
                            .map(|i| pool[i % pool.len()].clone())
                            .collect();
                    }
                }
                data.user_character_skill_tree_list
                    .push(UserCharacterSkillTree {
                        character_id: ch.clone(),
                        released_skill_tree_node_group_ids: released
                            .iter()
                            .filter(|r| r.character_id == ch)
                            .map(|r| r.node_group_id.clone())
                            .collect(),
                        connected_skill_tree_node_group_ids: groups,
                        connected_skill_tree_node_card_ids: cards,
                    });
            }
        }

        data.user_skill_tree_point_list = load_rows!(
            self,
            uid,
            SkillTreePointRow,
            "SELECT * FROM user_skill_tree_points WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserSkillTreePoint {
            skill_tree_point_id: r.skill_tree_point_id,
            quantity: r.quantity,
        })
        .collect();

        data.user_costume_list = load_rows!(
            self,
            uid,
            UserCostumeRow,
            "SELECT * FROM user_costumes WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserCostume {
            costume_id: r.costume_id,
            acquired_time: r.acquired_time,
            read_time: r.read_time,
        })
        .collect();

        data.user_sd_costume_list = load_rows!(
            self,
            uid,
            UserSdCostumeRow,
            "SELECT * FROM user_sd_costumes WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserSdCostume {
            sd_costume_id: r.sd_costume_id,
            acquired_time: r.acquired_time,
            read_time: r.read_time,
        })
        .collect();

        data.user_sd_costume_hair_accessory_list = load_rows!(
            self,
            uid,
            UserSdCostumeHairAccessoryRow,
            "SELECT * FROM user_sd_costume_hair_accessories WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserSdCostumeHairAccessory {
            sd_costume_hair_accessory_id: r.sd_costume_hair_accessory_id,
            read_time: r.read_time,
        })
        .collect();

        // missions with their reward thresholds
        {
            let thresholds: Vec<MissionThresholdRow> = load_rows!(
                self,
                uid,
                MissionThresholdRow,
                "SELECT * FROM user_mission_reward_thresholds WHERE account_id = ?"
            );
            let missions: Vec<UserMissionRow> = load_rows!(
                self,
                uid,
                UserMissionRow,
                "SELECT * FROM user_missions WHERE account_id = ?"
            );
            for r in missions {
                data.user_mission_list.push(UserMission {
                    mission_id: r.mission_id.clone(),
                    mission_pass_id: r.mission_pass_id,
                    progress: r.progress,
                    reward_received_thresholds: thresholds
                        .iter()
                        .filter(|t| t.mission_id == r.mission_id)
                        .map(|t| t.threshold)
                        .collect(),
                    last_progress_time: r.last_progress_time,
                    ttl_base_time: r.ttl_base_time,
                });
            }
        }

        // mission passes with received levels
        {
            let levels: Vec<MissionPassReceivedLevelRow> = load_rows!(
                self,
                uid,
                MissionPassReceivedLevelRow,
                "SELECT * FROM user_mission_pass_received_levels WHERE account_id = ?"
            );
            let passes: Vec<UserMissionPassRow> = load_rows!(
                self,
                uid,
                UserMissionPassRow,
                "SELECT * FROM user_mission_passes WHERE account_id = ?"
            );
            for r in passes {
                data.user_mission_pass_list.push(UserMissionPass {
                    mission_pass_id: r.mission_pass_id.clone(),
                    premium_pass_released_time: r.premium_pass_released_time,
                    normal_reward_received_levels: levels
                        .iter()
                        .filter(|l| l.mission_pass_id == r.mission_pass_id && !l.is_premium)
                        .map(|l| l.level)
                        .collect(),
                    premium_reward_received_levels: levels
                        .iter()
                        .filter(|l| l.mission_pass_id == r.mission_pass_id && l.is_premium)
                        .map(|l| l.level)
                        .collect(),
                    finished_time: r.finished_time,
                    received_daily_mission_pass_point_quantity: r
                        .received_daily_mission_pass_point_quantity,
                    received_weekly_mission_pass_point_quantity: r
                        .received_weekly_mission_pass_point_quantity,
                });
            }
        }

        data.user_mission_pass_point_list = load_rows!(
            self,
            uid,
            MissionPassPointRow,
            "SELECT * FROM user_mission_pass_points WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserMissionPassPoint {
            mission_pass_point_id: r.mission_pass_point_id,
            quantity: r.quantity,
        })
        .collect();

        data.user_notification_list = load_rows!(
            self,
            uid,
            UserNotificationRow,
            "SELECT * FROM user_notifications WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserNotification {
            notification_type: r.notification_type,
            is_active: r.is_active,
        })
        .collect();

        data.user_notification_read_time_list = load_rows!(
            self,
            uid,
            UserNotificationReadTimeRow,
            "SELECT * FROM user_notification_read_times WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserNotificationReadTime {
            notification_read_type: r.notification_read_type,
            read_time: r.read_time,
        })
        .collect();

        data.user_shop_charge_item_subscription_list = load_rows!(
            self,
            uid,
            UserMembershipRow,
            "SELECT * FROM user_memberships WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserShopChargeItemSubscription {
            shop_charge_item_subscription_shop_charge_item_id: r.shop_charge_item_id,
            status: r.status,
            purchased_time: r.purchased_time,
            expired_time: r.expired_time,
            is_auto_renew: r.is_auto_renew,
            total_join_month_count: r.total_join_month_count,
        })
        .collect();

        data.user_gacha_button_list = load_rows!(
            self,
            uid,
            UserGachaButtonRow,
            "SELECT * FROM user_gacha_buttons WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserGachaButton {
            gacha_button_id: r.gacha_button_id,
            draw_count: r.draw_count,
            last_draw_time: r.last_draw_time,
        })
        .collect();

        data.user_tutorial_list = load_rows!(
            self,
            uid,
            UserTutorialRow,
            "SELECT * FROM user_tutorials WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserTutorial {
            r#type: r.r#type,
            step: r.step,
        })
        .collect();

        data.user_stamp_list = load_rows!(
            self,
            uid,
            UserStampRow,
            "SELECT * FROM user_stamps WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserStamp {
            stamp_id: r.stamp_id,
            is_active: r.is_active,
        })
        .collect();

        data.user_emblem_list = load_rows!(
            self,
            uid,
            UserEmblemRow,
            "SELECT * FROM user_emblems WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserEmblem {
            emblem_id: r.emblem_id,
            last_activated_time: r.last_activated_time,
        })
        .collect();

        data.user_music_list = load_rows!(
            self,
            uid,
            UserMusicRow,
            "SELECT * FROM user_musics WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserMusic {
            music_id: r.music_id,
            is_favorite: r.is_favorite,
            released_time: r.released_time,
            highest_score: r.highest_score,
            highest_score_last_updated_time: r.highest_score_last_updated_time,
            highest_score_music_difficulty_type: r.highest_score_music_difficulty_type,
            highest_score_character_id: r.highest_score_character_id,
            highest_score_costume_id: r.highest_score_costume_id,
            received_highest_score_evaluation_rank_reward_rank_type: r
                .received_highest_score_evaluation_rank_reward_rank_type,
            ..Default::default()
        })
        .collect();

        data.user_music_difficulty_list = load_rows!(
            self,
            uid,
            UserMusicDifficultyRow,
            "SELECT * FROM user_music_difficulties WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserMusicDifficulty {
            music_id: r.music_id,
            difficulty_type: r.difficulty_type,
            highest_score: r.highest_score,
            non_highest_score_rating_character_highest_score: r
                .non_highest_score_rating_character_highest_score,
            non_highest_score_rating_character_highest_score_character_id: r
                .non_highest_score_rating_character_highest_score_character_id,
            max_combo_count: r.max_combo_count,
            clear_count: r.clear_count,
            live_result_type: r.live_result_type,
            technical_highest_score: r.technical_highest_score,
        })
        .collect();

        data.user_music_character_highest_score_list = load_rows!(
            self,
            uid,
            UserMusicCharacterHighestScoreRow,
            "SELECT * FROM user_music_character_highest_scores WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserMusicCharacterHighestScore {
            character_id: r.character_id,
            music_id: r.music_id,
            highest_score_infos: vec![HighestScoreInfo {
                difficulty_type: r.difficulty_type,
                highest_score: r.highest_score,
                highest_score_rating_value: r.highest_score_rating_value,
                highest_score_last_updated_time: r.highest_score_last_updated_time,
            }],
        })
        .collect();

        if let Some(r) =
            sqlx::query_as::<_, UserLiveRow>("SELECT * FROM user_lives WHERE account_id = ?")
                .bind(uid)
                .fetch_optional(self.pool())
                .await?
        {
            data.user_live = Some(UserLive {
                reward_up_stamina: r.reward_up_stamina,
                last_reward_up_stamina_auto_recovery_time: r
                    .last_reward_up_stamina_auto_recovery_time,
                reward_up_stamina_consumption_setting_quantity: r
                    .reward_up_stamina_consumption_setting_quantity,
                last_single_played_music_id: r.last_single_played_music_id,
                last_single_played_music_difficulty_type: r
                    .last_single_played_music_difficulty_type,
                last_multi_selected_music_id: r.last_multi_selected_music_id,
                last_multi_selected_music_difficulty_type: r
                    .last_multi_selected_music_difficulty_type,
                last_watched_live_deck_character_id: r.last_watched_live_deck_character_id,
                last_watched_live_deck_number: r.last_watched_live_deck_number,
                last_played_character_id: r.last_played_character_id,
            });
        }

        data.user_live_deck_list = load_rows!(
            self,
            uid,
            UserLiveDeckRow,
            "SELECT * FROM user_live_decks WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserLiveDeck {
            character_id: r.character_id,
            number: r.number,
            name: r.name,
            costume_id: r.costume_id,
        })
        .collect();

        data.user_live_deck_position_list = load_rows!(
            self,
            uid,
            UserLiveDeckPositionRow,
            "SELECT * FROM user_live_deck_positions WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserLiveDeckPosition {
            character_id: r.character_id,
            number: r.number,
            position: r.position,
            card_id: r.card_id,
        })
        .collect();

        data.user_live_skin_list = load_rows!(
            self,
            uid,
            UserLiveSkinRow,
            "SELECT * FROM user_live_skins WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserLiveSkin {
            live_skin_id: r.live_skin_id,
        })
        .collect();

        if let Some(r) =
            sqlx::query_as::<_, UserParkRow>("SELECT * FROM user_parks WHERE account_id = ?")
                .bind(uid)
                .fetch_optional(self.pool())
                .await?
        {
            let level_rewards: Vec<UserParkLevelRewardRow> = load_rows!(
                self,
                uid,
                UserParkLevelRewardRow,
                "SELECT * FROM user_park_level_rewards WHERE account_id = ?"
            );
            data.user_park = Some(UserPark {
                time_point: r.time_point,
                current_area_id: r.current_area_id,
                initial_park_character_id: r.initial_park_character_id,
                character_quest_last_picked_time: r.character_quest_last_picked_time,
                picked_daily_park_quest_id: r.picked_daily_park_quest_id,
                daily_quest_last_picked_time: r.daily_quest_last_picked_time,
                last_cleared_main_park_quest_id: r.last_cleared_main_park_quest_id,
                last_cleared_main_park_quest_step_group_number: r
                    .last_cleared_main_park_quest_step_group_number,
                last_cleared_main_park_quest_step: r.last_cleared_main_park_quest_step,
                received_player_level_reward_levels: level_rewards
                    .into_iter()
                    .map(|l| l.level)
                    .collect(),
                ..Default::default()
            });
        }

        data.user_park_quest_list = load_rows!(
            self,
            uid,
            UserParkQuestRow,
            "SELECT * FROM user_park_quests WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserParkQuest {
            park_quest_id: r.park_quest_id,
            start_time: r.start_time,
            is_progress: r.is_progress,
            clear_time: r.clear_time,
            clear_count: r.clear_count,
        })
        .collect();

        data.user_park_quest_step_list = load_rows!(
            self,
            uid,
            UserParkQuestStepRow,
            "SELECT * FROM user_park_quest_steps WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserParkQuestStep {
            park_quest_id: r.park_quest_id,
            step_group_number: r.step_group_number,
            current_step: r.current_step,
            all_clear_time: r.all_clear_time,
        })
        .collect();

        data.user_park_emotion_list = load_rows!(
            self,
            uid,
            UserParkEmotionRow,
            "SELECT * FROM user_park_emotions WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserParkEmotion {
            park_emotion_id: r.park_emotion_id,
            acquired_time: r.acquired_time,
        })
        .collect();

        data.user_park_accessory_list = load_rows!(
            self,
            uid,
            UserParkAccessoryRow,
            "SELECT * FROM user_park_accessories WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserParkAccessory {
            park_accessory_id: r.park_accessory_id,
        })
        .collect();

        data.user_park_area_selector_list = load_rows!(
            self,
            uid,
            UserParkAreaSelectorRow,
            "SELECT * FROM user_park_area_selectors WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserParkAreaSelector {
            area_selector_id: r.area_selector_id,
            selected_area_id: r.selected_area_id,
        })
        .collect();

        data.user_story_list = load_rows!(
            self,
            uid,
            UserStoryRow,
            "SELECT * FROM user_stories WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserStory {
            story_id: r.story_id,
            read_time: r.read_time,
        })
        .collect();

        data.user_wallpaper_list = load_rows!(
            self,
            uid,
            UserWallpaperRow,
            "SELECT * FROM user_wallpapers WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserWallpaper {
            wallpaper_id: r.wallpaper_id,
        })
        .collect();

        data.user_instant_tips_list = load_rows!(
            self,
            uid,
            UserInstantTipRow,
            "SELECT * FROM user_instant_tips WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserInstantTips {
            instant_tips_id: r.instant_tips_id,
        })
        .collect();

        data.user_facility_list = load_rows!(
            self,
            uid,
            UserFacilityRow,
            "SELECT * FROM user_facilities WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserFacility {
            facility_id: r.facility_id,
            current_level: r.current_level,
        })
        .collect();

        data.user_fan_mark_list = load_rows!(
            self,
            uid,
            UserFanMarkRow,
            "SELECT * FROM user_fan_marks WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserFanMark {
            fan_mark_id: r.fan_mark_id,
            acquired_time: r.acquired_time,
        })
        .collect();

        data.user_poster_list = load_rows!(
            self,
            uid,
            UserPosterRow,
            "SELECT * FROM user_posters WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| UserPoster {
            poster_id: r.poster_id,
        })
        .collect();

        data.user_custom_palette_list = load_rows!(
            self,
            uid,
            UserCustomPaletteRow,
            "SELECT * FROM user_custom_palettes WHERE account_id = ? ORDER BY number"
        )
        .into_iter()
        .map(|r| UserCustomPalette {
            number: r.number,
            image_url: r.image_url,
            is_inactivated: r.is_inactivated,
        })
        .collect();

        data.user_jump_rope_list = {
            let mut list = Vec::new();
            for r in load_rows!(
                self,
                uid,
                UserJumpRopeRow,
                "SELECT * FROM user_jump_ropes WHERE account_id = ?"
            ) {
                let npc_exits: Vec<i32> = sqlx::query_scalar(
                    "SELECT jump_count FROM user_jump_rope_npc_exits WHERE account_id = ? AND jump_rope_id = ? ORDER BY exit_index",
                )
                .bind(uid)
                .bind(&r.jump_rope_id)
                .fetch_all(self.pool())
                .await?;
                list.push(UserJumpRope {
                    jump_rope_id: r.jump_rope_id,
                    best_jump_count: r.best_jump_count,
                    is_cleared: r.is_cleared,
                    play_count: r.play_count as i64,
                    notification_read_time: r.notification_read_time,
                    npc_exit_jump_counts: npc_exits,
                });
            }
            list
        };

        data.user_exchange_booth_list = load_rows!(
            self,
            uid,
            UserExchangeBoothRow,
            "SELECT * FROM user_exchange_booths WHERE account_id = ?"
        )
        .into_iter()
        .map(|r| {
            let booth_id = r.exchange_booth_id.clone();
            UserExchangeBooth {
                exchange_booth_id: r.exchange_booth_id,
                exchange_booth_fixed_item_ids: match ExchangeBoothFixedItem::try_table() {
                    Some(rows) => rows
                        .iter()
                        .filter(|i| i.exchange_booth_id == booth_id)
                        .map(|i| i.id.clone())
                        .collect(),
                    None => vec![],
                },
                purchased_fixed_quantities: vec![],
                fixed_item_release_times: vec![],
                lottery_item_release_times: vec![],
                last_reset_time: 0,
                read_time: r.last_read_time,
            }
        })
        .collect();

        data.user_payment_charge_order_list = match ShopChargeItemProduct::try_table() {
            Some(rows) => {
                let mut ids: Vec<String> =
                    rows.iter().map(|p| p.shop_charge_item_id.clone()).collect();
                ids.sort();
                ids.dedup();
                ids.into_iter()
                    .map(|shop_charge_item_id| UserPaymentChargeOrder {
                        shop_charge_item_id,
                    })
                    .collect()
            }
            None => vec![],
        };

        Ok(data)
    }
}

// silence unused-import warnings for types used only in field construction
#[allow(unused_imports)]
use Sqlite as _Sqlite;
