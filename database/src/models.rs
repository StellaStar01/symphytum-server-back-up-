use sqlx::FromRow;

#[derive(FromRow, Clone)]
pub struct AccountRow {
    pub id: String,
    pub credential: String,
    pub created_at: i64,
    pub last_login_at: i64,
}

#[derive(FromRow, Clone, Default)]
pub struct UserRow {
    pub account_id: String,
    pub region: i32,
    pub country_code: String,
    pub birth_year_for_payment: i32,
    pub birth_month_for_payment: i32,
    pub active_user_type: i32,
    pub tutorial_cleared_time: i64,
    pub deleted_time: i64,
}

#[derive(FromRow, Clone, Default)]
pub struct UserProfileRow {
    pub account_id: String,
    pub name: String,
    pub message: String,
    pub park_character_id: String,
    pub fan_mark_id: String,
    pub is_public_user_id_publish: bool,
    pub is_basic_info_publish: bool,
    pub is_character_rank_publish: bool,
    pub is_live_result_publish: bool,
    pub is_mini_game_result_publish: bool,
    pub is_user_info_publish_in_multi_game: bool,
    pub custom_palette_number: i32,
    pub exp: i64,
    pub highest_live_deck_evaluation_value: i64,
    pub highest_live_deck_evaluation_character_id: String,
    pub highest_live_deck_evaluation_costume_id: String,
    pub name_last_updated_time: i64,
    pub is_official: bool,
    pub login_status_last_updated_time: i64,
    pub multi_game_unpublished_user_name: String,
    pub message_last_updated_time: i64,
}

#[derive(FromRow, Clone)]
pub struct ProfileDeckCardRow {
    pub account_id: String,
    pub position: i32,
    pub card_id: String,
    pub level: i32,
    pub potential_upgrade_count: i32,
}

#[derive(FromRow, Clone)]
pub struct EmblemPositionRow {
    pub account_id: String,
    pub position: i32,
    pub emblem_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserTimeRow {
    pub account_id: String,
    pub game_start_time: i64,
    pub last_login_time: i64,
    pub force_relogin_flag_set_time: i64,
    pub last_login_os: i32,
    pub comebacked_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserCountRow {
    pub account_id: String,
    pub count_type: i32,
    pub total_count: i64,
    pub daily_count: i64,
    pub last_updated_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserBalanceRow {
    pub account_id: String,
    pub free_quantity: i32,
    pub paid_quantity: i32,
}

#[derive(FromRow, Clone)]
pub struct UserItemRow {
    pub account_id: String,
    pub item_id: String,
    pub expired_time: i64,
    pub quantity: i64,
    pub last_acquired_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserItemOwnedRow {
    pub account_id: String,
    pub item_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserCardRow {
    pub account_id: String,
    pub card_id: String,
    pub exp: i64,
    pub level_limit_break_count: i32,
    pub potential_upgrade_count: i32,
    pub potential_upgrade_point_quantity: i32,
    pub acquired_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserCharacterRow {
    pub account_id: String,
    pub character_id: String,
    pub costume_id: String,
    pub sd_costume_id: String,
    pub sd_costume_hair_accessory_id: String,
    pub exp: i64,
    pub highest_live_deck_evaluation_value: i64,
    pub acquired_time: i64,
    pub last_reward_received_level: i32,
    pub read_time: i64,
    pub park_read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct SkillTreeEntryRow {
    pub account_id: String,
    pub character_id: String,
    pub node_group_id: String,
}

#[derive(FromRow, Clone)]
pub struct SkillTreeCardRow {
    pub account_id: String,
    pub character_id: String,
    pub card_id: String,
}

#[derive(FromRow, Clone)]
pub struct SkillTreePointRow {
    pub account_id: String,
    pub skill_tree_point_id: String,
    pub quantity: i64,
}

#[derive(FromRow, Clone)]
pub struct UserCostumeRow {
    pub account_id: String,
    pub costume_id: String,
    pub acquired_time: i64,
    pub read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserSdCostumeRow {
    pub account_id: String,
    pub sd_costume_id: String,
    pub acquired_time: i64,
    pub read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserSdCostumeHairAccessoryRow {
    pub account_id: String,
    pub sd_costume_hair_accessory_id: String,
    pub read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserMissionRow {
    pub account_id: String,
    pub mission_id: String,
    pub mission_pass_id: String,
    pub progress: i64,
    pub last_progress_time: i64,
    pub ttl_base_time: i64,
}

#[derive(FromRow, Clone)]
pub struct MissionThresholdRow {
    pub account_id: String,
    pub mission_id: String,
    pub threshold: i64,
}

#[derive(FromRow, Clone)]
pub struct UserMissionPassRow {
    pub account_id: String,
    pub mission_pass_id: String,
    pub premium_pass_released_time: i64,
    pub finished_time: i64,
    pub received_daily_mission_pass_point_quantity: i64,
    pub received_weekly_mission_pass_point_quantity: i64,
}

#[derive(FromRow, Clone)]
pub struct MissionPassReceivedLevelRow {
    pub account_id: String,
    pub mission_pass_id: String,
    pub level: i32,
    pub is_premium: bool,
}

#[derive(FromRow, Clone)]
pub struct MissionPassPointRow {
    pub account_id: String,
    pub mission_pass_point_id: String,
    pub quantity: i64,
}

#[derive(FromRow, Clone)]
pub struct UserNotificationRow {
    pub account_id: String,
    pub notification_type: i32,
    pub is_active: bool,
}

#[derive(FromRow, Clone)]
pub struct UserNotificationReadTimeRow {
    pub account_id: String,
    pub notification_read_type: i32,
    pub read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserMembershipRow {
    pub account_id: String,
    pub shop_charge_item_id: String,
    pub status: i32,
    pub purchased_time: i64,
    pub expired_time: i64,
    pub is_auto_renew: bool,
    pub total_join_month_count: i32,
}

#[derive(FromRow, Clone)]
pub struct UserGachaButtonRow {
    pub account_id: String,
    pub gacha_button_id: String,
    pub draw_count: i32,
    pub last_draw_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserTutorialRow {
    pub account_id: String,
    pub r#type: i32,
    pub step: i32,
}

#[derive(FromRow, Clone)]
pub struct UserStampRow {
    pub account_id: String,
    pub stamp_id: String,
    pub is_active: bool,
}

#[derive(FromRow, Clone)]
pub struct UserEmblemRow {
    pub account_id: String,
    pub emblem_id: String,
    pub last_activated_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserMusicRow {
    pub account_id: String,
    pub music_id: String,
    pub is_favorite: bool,
    pub released_time: i64,
    pub highest_score: i64,
    pub highest_score_last_updated_time: i64,
    pub highest_score_music_difficulty_type: i32,
    pub highest_score_character_id: String,
    pub highest_score_costume_id: String,
    pub received_highest_score_evaluation_rank_reward_rank_type: i32,
}

#[derive(FromRow, Clone)]
pub struct UserMusicDifficultyRow {
    pub account_id: String,
    pub music_id: String,
    pub difficulty_type: i32,
    pub highest_score: i64,
    pub non_highest_score_rating_character_highest_score: i64,
    pub non_highest_score_rating_character_highest_score_character_id: String,
    pub max_combo_count: i64,
    pub clear_count: i64,
    pub live_result_type: i32,
    pub technical_highest_score: i64,
}

#[derive(FromRow, Clone)]
pub struct UserMusicCharacterHighestScoreRow {
    pub account_id: String,
    pub character_id: String,
    pub music_id: String,
    pub difficulty_type: i32,
    pub highest_score: i64,
    pub highest_score_rating_value: i64,
    pub highest_score_last_updated_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserLiveRow {
    pub account_id: String,
    pub reward_up_stamina: i32,
    pub last_reward_up_stamina_auto_recovery_time: i64,
    pub reward_up_stamina_consumption_setting_quantity: i64,
    pub last_single_played_music_id: String,
    pub last_single_played_music_difficulty_type: i32,
    pub last_multi_selected_music_id: String,
    pub last_multi_selected_music_difficulty_type: i32,
    pub last_watched_live_deck_character_id: String,
    pub last_watched_live_deck_number: i32,
    pub last_played_character_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserLiveDeckRow {
    pub account_id: String,
    pub character_id: String,
    pub number: i32,
    pub name: String,
    pub costume_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserLiveDeckPositionRow {
    pub account_id: String,
    pub character_id: String,
    pub number: i32,
    pub position: i32,
    pub card_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserLiveSkinRow {
    pub account_id: String,
    pub live_skin_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserParkRow {
    pub account_id: String,
    pub time_point: i32,
    pub current_area_id: String,
    pub initial_park_character_id: String,
    pub character_quest_last_picked_time: i64,
    pub picked_daily_park_quest_id: String,
    pub daily_quest_last_picked_time: i64,
    pub last_cleared_main_park_quest_id: String,
    pub last_cleared_main_park_quest_step_group_number: i64,
    pub last_cleared_main_park_quest_step: i64,
}

#[derive(FromRow, Clone)]
pub struct UserParkLevelRewardRow {
    pub account_id: String,
    pub level: i32,
}

#[derive(FromRow, Clone)]
pub struct UserParkQuestRow {
    pub account_id: String,
    pub park_quest_id: String,
    pub start_time: i64,
    pub is_progress: bool,
    pub clear_time: i64,
    pub clear_count: i64,
}

#[derive(FromRow, Clone)]
pub struct UserParkQuestStepRow {
    pub account_id: String,
    pub park_quest_id: String,
    pub step_group_number: i32,
    pub current_step: i32,
    pub all_clear_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserParkEmotionRow {
    pub account_id: String,
    pub park_emotion_id: String,
    pub acquired_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserParkAccessoryRow {
    pub account_id: String,
    pub park_accessory_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserParkAreaSelectorRow {
    pub account_id: String,
    pub area_selector_id: String,
    pub selected_area_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserStoryRow {
    pub account_id: String,
    pub story_id: String,
    pub read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserWallpaperRow {
    pub account_id: String,
    pub wallpaper_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserInstantTipRow {
    pub account_id: String,
    pub instant_tips_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserFacilityRow {
    pub account_id: String,
    pub facility_id: String,
    pub current_level: i32,
}

#[derive(FromRow, Clone)]
pub struct UserFanMarkRow {
    pub account_id: String,
    pub fan_mark_id: String,
    pub acquired_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserPosterRow {
    pub account_id: String,
    pub poster_id: String,
}

#[derive(FromRow, Clone)]
pub struct UserAppReviewRow {
    pub account_id: String,
    pub is_displayed: bool,
}

#[derive(FromRow, Clone, Default)]
pub struct UserJumpRopeRow {
    pub account_id: String,
    pub jump_rope_id: String,
    pub best_jump_count: i64,
    pub is_cleared: bool,
    pub play_count: i32,
    pub notification_read_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserJumpRopeNpcExitRow {
    pub account_id: String,
    pub jump_rope_id: String,
    pub exit_index: i32,
    pub jump_count: i32,
}

#[derive(FromRow, Clone, Default)]
pub struct UserExchangeBoothRow {
    pub account_id: String,
    pub exchange_booth_id: String,
    pub last_read_time: i64,
}

#[derive(FromRow, Clone, Default)]
pub struct UserExchangeBoothPurchaseRow {
    pub account_id: String,
    pub exchange_booth_id: String,
    pub booth_item_id: String,
    pub purchased_count: i64,
    pub last_purchased_time: i64,
}

#[derive(FromRow, Clone)]
pub struct UserNoticeReadTimeRow {
    pub account_id: String,
    pub notice_id: String,
    pub read_time: i64,
}

#[derive(FromRow, Clone, Default)]
pub struct UserCustomPaletteRow {
    pub account_id: String,
    pub number: i32,
    pub image_url: String,
    pub is_inactivated: bool,
    pub background_card_id: String,
    pub background_card_potential_upgrade_count: i32,
}

#[derive(FromRow, Clone, Default)]
pub struct UserCustomPalettePartRow {
    pub account_id: String,
    pub number: i32,
    pub part_index: i32,
    pub resource_type: i32,
    pub resource_id: String,
    pub position_x_permil: i32,
    pub position_y_permil: i32,
    pub scale_permil: i32,
    pub rotation_permil: i32,
    pub layer: i32,
}
