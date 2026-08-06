CREATE TABLE accounts (
    id            TEXT PRIMARY KEY,
    credential    TEXT NOT NULL UNIQUE,
    created_at    INTEGER NOT NULL,
    last_login_at INTEGER NOT NULL
);

CREATE TABLE users (
    account_id               TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    region                   INTEGER NOT NULL DEFAULT 1,
    country_code             TEXT NOT NULL DEFAULT 'JP',
    birth_year_for_payment   INTEGER NOT NULL DEFAULT 0,
    birth_month_for_payment  INTEGER NOT NULL DEFAULT 0,
    active_user_type         INTEGER NOT NULL DEFAULT 3,
    tutorial_cleared_time    INTEGER NOT NULL DEFAULT 0,
    deleted_time             INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE user_profiles (
    account_id                          TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    name                                TEXT NOT NULL DEFAULT '',
    message                             TEXT NOT NULL DEFAULT '',
    park_character_id                   TEXT NOT NULL DEFAULT '',
    fan_mark_id                         TEXT NOT NULL DEFAULT '',
    is_public_user_id_publish           INTEGER NOT NULL DEFAULT 0,
    is_basic_info_publish               INTEGER NOT NULL DEFAULT 0,
    is_character_rank_publish           INTEGER NOT NULL DEFAULT 0,
    is_live_result_publish              INTEGER NOT NULL DEFAULT 0,
    is_mini_game_result_publish         INTEGER NOT NULL DEFAULT 0,
    is_user_info_publish_in_multi_game  INTEGER NOT NULL DEFAULT 0,
    custom_palette_number               INTEGER NOT NULL DEFAULT 0,
    exp                                 INTEGER NOT NULL DEFAULT 0,
    highest_live_deck_evaluation_value  INTEGER NOT NULL DEFAULT 0,
    highest_live_deck_evaluation_character_id TEXT NOT NULL DEFAULT '',
    highest_live_deck_evaluation_costume_id   TEXT NOT NULL DEFAULT '',
    name_last_updated_time              INTEGER NOT NULL DEFAULT 0,
    is_official                         INTEGER NOT NULL DEFAULT 0,
    login_status_last_updated_time      INTEGER NOT NULL DEFAULT 0,
    multi_game_unpublished_user_name    TEXT NOT NULL DEFAULT '',
    message_last_updated_time           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE user_profile_highest_deck_cards (
    account_id            TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    position              INTEGER NOT NULL,
    card_id               TEXT NOT NULL,
    level                 INTEGER NOT NULL,
    potential_upgrade_count INTEGER NOT NULL,
    PRIMARY KEY (account_id, position)
);

CREATE TABLE user_profile_emblem_positions (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    position   INTEGER NOT NULL,
    emblem_id  TEXT NOT NULL,
    PRIMARY KEY (account_id, position)
);

CREATE TABLE user_times (
    account_id                    TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    game_start_time               INTEGER NOT NULL,
    last_login_time               INTEGER NOT NULL,
    force_relogin_flag_set_time   INTEGER NOT NULL DEFAULT 0,
    last_login_os                 INTEGER NOT NULL DEFAULT 3,
    comebacked_time               INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE user_counts (
    account_id        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    count_type        INTEGER NOT NULL,
    total_count       INTEGER NOT NULL DEFAULT 0,
    daily_count       INTEGER NOT NULL DEFAULT 0,
    last_updated_time INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, count_type)
);

CREATE TABLE user_balances (
    account_id    TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    free_quantity INTEGER NOT NULL DEFAULT 0,
    paid_quantity INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE user_items (
    account_id         TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    item_id            TEXT NOT NULL,
    expired_time       INTEGER NOT NULL DEFAULT 0,
    quantity           INTEGER NOT NULL DEFAULT 0,
    last_acquired_time INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, item_id, expired_time)
);

CREATE TABLE user_item_owned (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    item_id    TEXT NOT NULL,
    PRIMARY KEY (account_id, item_id)
);

CREATE TABLE user_cards (
    account_id                       TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    card_id                          TEXT NOT NULL,
    exp                              INTEGER NOT NULL DEFAULT 0,
    level_limit_break_count          INTEGER NOT NULL DEFAULT 0,
    potential_upgrade_count          INTEGER NOT NULL DEFAULT 0,
    potential_upgrade_point_quantity INTEGER NOT NULL DEFAULT 0,
    acquired_time                    INTEGER NOT NULL,
    PRIMARY KEY (account_id, card_id)
);

CREATE TABLE user_characters (
    account_id                        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id                      TEXT NOT NULL,
    costume_id                        TEXT NOT NULL DEFAULT '',
    sd_costume_id                     TEXT NOT NULL DEFAULT '',
    sd_costume_hair_accessory_id      TEXT NOT NULL DEFAULT '',
    exp                               INTEGER NOT NULL DEFAULT 0,
    highest_live_deck_evaluation_value INTEGER NOT NULL DEFAULT 0,
    acquired_time                     INTEGER NOT NULL,
    last_reward_received_level        INTEGER NOT NULL DEFAULT 1,
    read_time                         INTEGER NOT NULL DEFAULT 0,
    park_read_time                    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, character_id)
);

CREATE TABLE user_character_skill_tree_released (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id  TEXT NOT NULL,
    node_group_id TEXT NOT NULL,
    PRIMARY KEY (account_id, character_id, node_group_id)
);

CREATE TABLE user_character_skill_tree_connected (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id  TEXT NOT NULL,
    node_group_id TEXT NOT NULL,
    PRIMARY KEY (account_id, character_id, node_group_id)
);

CREATE TABLE user_character_skill_tree_connected_cards (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id TEXT NOT NULL,
    card_id      TEXT NOT NULL,
    PRIMARY KEY (account_id, character_id, card_id)
);

CREATE TABLE user_skill_tree_points (
    account_id          TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    skill_tree_point_id TEXT NOT NULL,
    quantity            INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, skill_tree_point_id)
);

CREATE TABLE user_costumes (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    costume_id    TEXT NOT NULL,
    acquired_time INTEGER NOT NULL,
    read_time     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, costume_id)
);

CREATE TABLE user_sd_costumes (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    sd_costume_id TEXT NOT NULL,
    acquired_time INTEGER NOT NULL,
    read_time     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, sd_costume_id)
);

CREATE TABLE user_sd_costume_hair_accessories (
    account_id                   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    sd_costume_hair_accessory_id TEXT NOT NULL,
    read_time                    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, sd_costume_hair_accessory_id)
);

CREATE TABLE user_missions (
    account_id         TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_id         TEXT NOT NULL,
    mission_pass_id    TEXT NOT NULL DEFAULT '',
    progress           INTEGER NOT NULL DEFAULT 0,
    last_progress_time INTEGER NOT NULL DEFAULT 0,
    ttl_base_time      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, mission_id)
);

CREATE TABLE user_mission_reward_thresholds (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_id TEXT NOT NULL,
    threshold  INTEGER NOT NULL,
    PRIMARY KEY (account_id, mission_id, threshold)
);

CREATE TABLE user_mission_passes (
    account_id                                  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_pass_id                             TEXT NOT NULL,
    premium_pass_released_time                  INTEGER NOT NULL DEFAULT 0,
    finished_time                               INTEGER NOT NULL DEFAULT 0,
    received_daily_mission_pass_point_quantity  INTEGER NOT NULL DEFAULT 0,
    received_weekly_mission_pass_point_quantity INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, mission_pass_id)
);

CREATE TABLE user_mission_pass_received_levels (
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_pass_id TEXT NOT NULL,
    level           INTEGER NOT NULL,
    is_premium      INTEGER NOT NULL,
    PRIMARY KEY (account_id, mission_pass_id, level, is_premium)
);

CREATE TABLE user_mission_pass_points (
    account_id            TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_pass_point_id TEXT NOT NULL,
    quantity              INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, mission_pass_point_id)
);

CREATE TABLE user_notifications (
    account_id        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    notification_type INTEGER NOT NULL,
    is_active         INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, notification_type)
);

CREATE TABLE user_notification_read_times (
    account_id             TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    notification_read_type INTEGER NOT NULL,
    read_time              INTEGER NOT NULL,
    PRIMARY KEY (account_id, notification_read_type)
);

CREATE TABLE user_memberships (
    account_id              TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    shop_charge_item_id     TEXT NOT NULL,
    status                  INTEGER NOT NULL DEFAULT 1,
    purchased_time          INTEGER NOT NULL DEFAULT 0,
    expired_time            INTEGER NOT NULL DEFAULT 0,
    is_auto_renew           INTEGER NOT NULL DEFAULT 1,
    total_join_month_count  INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (account_id, shop_charge_item_id)
);

CREATE TABLE user_gacha_buttons (
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    gacha_button_id TEXT NOT NULL,
    draw_count      INTEGER NOT NULL DEFAULT 0,
    last_draw_time  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, gacha_button_id)
);

CREATE TABLE user_tutorials (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    type       INTEGER NOT NULL,
    step       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, type)
);

CREATE TABLE user_stamps (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    stamp_id   TEXT NOT NULL,
    is_active  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, stamp_id)
);

CREATE TABLE user_emblems (
    account_id          TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    emblem_id           TEXT NOT NULL,
    last_activated_time INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, emblem_id)
);

CREATE TABLE user_musics (
    account_id                                        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    music_id                                          TEXT NOT NULL,
    is_favorite                                       INTEGER NOT NULL DEFAULT 0,
    released_time                                     INTEGER NOT NULL DEFAULT 0,
    highest_score                                     INTEGER NOT NULL DEFAULT 0,
    highest_score_last_updated_time                   INTEGER NOT NULL DEFAULT 0,
    highest_score_music_difficulty_type               INTEGER NOT NULL DEFAULT 0,
    highest_score_character_id                        TEXT NOT NULL DEFAULT '',
    highest_score_costume_id                          TEXT NOT NULL DEFAULT '',
    received_highest_score_evaluation_rank_reward_rank_type INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, music_id)
);

CREATE TABLE user_music_difficulties (
    account_id         TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    music_id           TEXT NOT NULL,
    difficulty_type    INTEGER NOT NULL,
    highest_score      INTEGER NOT NULL DEFAULT 0,
    non_highest_score_rating_character_highest_score        INTEGER NOT NULL DEFAULT 0,
    non_highest_score_rating_character_highest_score_character_id TEXT NOT NULL DEFAULT '',
    max_combo_count    INTEGER NOT NULL DEFAULT 0,
    clear_count        INTEGER NOT NULL DEFAULT 0,
    live_result_type   INTEGER NOT NULL DEFAULT 0,
    technical_highest_score INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, music_id, difficulty_type)
);

CREATE TABLE user_music_character_highest_scores (
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id    TEXT NOT NULL,
    music_id        TEXT NOT NULL,
    difficulty_type INTEGER NOT NULL,
    highest_score   INTEGER NOT NULL DEFAULT 0,
    highest_score_rating_value INTEGER NOT NULL DEFAULT 0,
    highest_score_last_updated_time INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, character_id, music_id, difficulty_type)
);

CREATE TABLE user_lives (
    account_id                              TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    reward_up_stamina                       INTEGER NOT NULL DEFAULT 0,
    last_reward_up_stamina_auto_recovery_time INTEGER NOT NULL DEFAULT 0,
    reward_up_stamina_consumption_setting_quantity INTEGER NOT NULL DEFAULT 0,
    last_single_played_music_id             TEXT NOT NULL DEFAULT '',
    last_single_played_music_difficulty_type INTEGER NOT NULL DEFAULT 0,
    last_multi_selected_music_id            TEXT NOT NULL DEFAULT '',
    last_multi_selected_music_difficulty_type INTEGER NOT NULL DEFAULT 0,
    last_watched_live_deck_character_id     TEXT NOT NULL DEFAULT '',
    last_watched_live_deck_number           INTEGER NOT NULL DEFAULT 0,
    last_played_character_id                TEXT NOT NULL DEFAULT ''
);

CREATE TABLE user_live_decks (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id  TEXT NOT NULL,
    number        INTEGER NOT NULL,
    name          TEXT NOT NULL DEFAULT '',
    costume_id    TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (account_id, character_id, number)
);

CREATE TABLE user_live_deck_positions (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    character_id TEXT NOT NULL,
    number       INTEGER NOT NULL,
    position     INTEGER NOT NULL,
    card_id      TEXT NOT NULL,
    PRIMARY KEY (account_id, character_id, number, position)
);

CREATE TABLE user_live_skins (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    live_skin_id  TEXT NOT NULL,
    PRIMARY KEY (account_id, live_skin_id)
);

CREATE TABLE user_parks (
    account_id                        TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    time_point                        INTEGER NOT NULL DEFAULT 0,
    current_area_id                   TEXT NOT NULL DEFAULT '',
    initial_park_character_id         TEXT NOT NULL DEFAULT '',
    character_quest_last_picked_time  INTEGER NOT NULL DEFAULT 0,
    picked_daily_park_quest_id        TEXT NOT NULL DEFAULT '',
    daily_quest_last_picked_time      INTEGER NOT NULL DEFAULT 0,
    last_cleared_main_park_quest_id   TEXT NOT NULL DEFAULT '',
    last_cleared_main_park_quest_step_group_number INTEGER NOT NULL DEFAULT 0,
    last_cleared_main_park_quest_step INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE user_park_quests (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    park_quest_id TEXT NOT NULL,
    start_time    INTEGER NOT NULL DEFAULT 0,
    is_progress   INTEGER NOT NULL DEFAULT 0,
    clear_time    INTEGER NOT NULL DEFAULT 0,
    clear_count   INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, park_quest_id)
);

CREATE TABLE user_park_quest_steps (
    account_id        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    park_quest_id     TEXT NOT NULL,
    step_group_number INTEGER NOT NULL,
    current_step      INTEGER NOT NULL DEFAULT 0,
    all_clear_time    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, park_quest_id, step_group_number)
);

CREATE TABLE user_park_emotions (
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    park_emotion_id TEXT NOT NULL,
    acquired_time   INTEGER NOT NULL,
    PRIMARY KEY (account_id, park_emotion_id)
);

CREATE TABLE user_park_accessories (
    account_id        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    park_accessory_id TEXT NOT NULL,
    PRIMARY KEY (account_id, park_accessory_id)
);

CREATE TABLE user_park_area_selectors (
    account_id       TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    area_selector_id TEXT NOT NULL,
    selected_area_id TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (account_id, area_selector_id)
);

CREATE TABLE user_stories (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    story_id   TEXT NOT NULL,
    read_time  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, story_id)
);

CREATE TABLE user_wallpapers (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    wallpaper_id TEXT NOT NULL,
    PRIMARY KEY (account_id, wallpaper_id)
);

CREATE TABLE user_instant_tips (
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    instant_tips_id TEXT NOT NULL,
    PRIMARY KEY (account_id, instant_tips_id)
);

CREATE TABLE user_facilities (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    facility_id   TEXT NOT NULL,
    current_level INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (account_id, facility_id)
);

CREATE TABLE user_fan_marks (
    account_id    TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    fan_mark_id   TEXT NOT NULL,
    acquired_time INTEGER NOT NULL,
    PRIMARY KEY (account_id, fan_mark_id)
);

CREATE TABLE user_posters (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    poster_id  TEXT NOT NULL,
    PRIMARY KEY (account_id, poster_id)
);

CREATE TABLE user_app_reviews (
    account_id   TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    is_displayed INTEGER NOT NULL DEFAULT 0
);
