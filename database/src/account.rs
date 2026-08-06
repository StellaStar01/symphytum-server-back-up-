use resource::master::{MasterTable, load};
use sqlx::{Error, Sqlite, Transaction};
use std::collections::{BTreeMap, HashMap};
use types::entity::master::{
    AreaSelector, Card, CardLevel, CardLevelLimit, CardPotential, Character, Costume, Emblem,
    Facility, FanMark, InstantTips, Item, LiveSkin, Membership, Mission, MissionPass,
    MissionPassLevel, MissionPassPoint, MissionProgress, Music, ParkAccessory, ParkEmotion,
    ParkQuest, ParkQuestStep, PlayerLevel, Poster, SdCostume, SdCostumeHairAccessory,
    SkillTreeNode, SkillTreePoint, Stamp, Story, Tutorial, Wallpaper,
};

use crate::models::*;
use crate::unix_now;

use super::Database;

fn new_uid() -> String {
    let bytes: [u8; 16] = rand::random();
    let mut s = String::with_capacity(36);
    for (i, b) in bytes.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            s.push('-');
        }
        s.push_str(&format!("{b:02x}"));
    }
    s
}

impl Database {
    pub async fn get_or_create_account(&self, credential: &str) -> Result<String, Error> {
        let existing: Option<AccountRow> =
            sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE credential = ?")
                .bind(credential)
                .fetch_optional(self.pool())
                .await?;

        if let Some(acc) = existing {
            let now = unix_now();
            sqlx::query("UPDATE accounts SET last_login_at = ? WHERE id = ?")
                .bind(now)
                .bind(&acc.id)
                .execute(self.pool())
                .await?;
            sqlx::query("UPDATE user_times SET last_login_time = ? WHERE account_id = ?")
                .bind(now)
                .bind(&acc.id)
                .execute(self.pool())
                .await?;
            return Ok(acc.id);
        }

        let uid = new_uid();
        let now = unix_now();
        let mut tx = self.pool().begin().await?;

        sqlx::query(
            "INSERT INTO accounts (id, credential, created_at, last_login_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&uid)
        .bind(credential)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        self.max_out(&mut tx, &uid)
            .await
            .map_err(|e| Error::Protocol(format!("max_out failed: {e}")))?;

        tx.commit().await?;
        Ok(uid)
    }

    async fn max_out<'e>(&self, tx: &mut Transaction<'e, Sqlite>, uid: &str) -> Result<(), String> {
        let now = unix_now();

        // load the master tables max_out reads from
        for t in [
            load::<PlayerLevel>().await,
            load::<Card>().await,
            load::<CardLevel>().await,
            load::<CardLevelLimit>().await,
            load::<CardPotential>().await,
            load::<Character>().await,
            load::<Costume>().await,
            load::<SdCostume>().await,
            load::<SdCostumeHairAccessory>().await,
            load::<SkillTreeNode>().await,
            load::<SkillTreePoint>().await,
            load::<Mission>().await,
            load::<MissionProgress>().await,
            load::<MissionPass>().await,
            load::<MissionPassLevel>().await,
            load::<MissionPassPoint>().await,
            load::<Membership>().await,
            load::<Tutorial>().await,
            load::<Item>().await,
            load::<Stamp>().await,
            load::<Emblem>().await,
            load::<Music>().await,
            load::<LiveSkin>().await,
            load::<Wallpaper>().await,
            load::<ParkEmotion>().await,
            load::<ParkAccessory>().await,
            load::<AreaSelector>().await,
            load::<Story>().await,
            load::<Poster>().await,
            load::<InstantTips>().await,
            load::<Facility>().await,
            load::<FanMark>().await,
            load::<ParkQuest>().await,
            load::<ParkQuestStep>().await,
        ] {
            t?;
        }

        let max_level_row = PlayerLevel::table()
            .iter()
            .max_by_key(|r| r.level)
            .expect("PlayerLevel non-empty");
        sqlx::query(
            "INSERT INTO users (account_id, region, country_code, active_user_type, tutorial_cleared_time) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uid).bind(1i32).bind("JP").bind(3i32).bind(now)
        .execute(&mut **tx).await.map_err(|e| e.to_string())?;

        sqlx::query("INSERT INTO user_profiles (account_id, name) VALUES (?, ?)")
            .bind(uid)
            .bind("Maxed")
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO user_times (account_id, game_start_time, last_login_time, last_login_os) VALUES (?, ?, ?, ?)",
        )
        .bind(uid).bind(now).bind(now).bind(3i32)
        .execute(&mut **tx).await.map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO user_balances (account_id, free_quantity, paid_quantity) VALUES (?, ?, ?)",
        )
        .bind(uid)
        .bind(999_999i32)
        .bind(999_999i32)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("INSERT INTO user_app_reviews (account_id, is_displayed) VALUES (?, ?)")
            .bind(uid)
            .bind(1i32)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;

        // max player level exp
        sqlx::query("UPDATE user_profiles SET exp = ? WHERE account_id = ?")
            .bind(max_level_row.exp)
            .bind(uid)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO user_counts (account_id, count_type, total_count, daily_count, last_updated_time) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uid).bind(1i32).bind(1i64).bind(1i64).bind(now)
        .execute(&mut **tx).await.map_err(|e| e.to_string())?;

        for item in Item::table() {
            let qty = if item.r#type == 0 { 999 } else { 999 };
            sqlx::query(
                "INSERT OR IGNORE INTO user_items (account_id, item_id, expired_time, quantity, last_acquired_time) VALUES (?, ?, 0, ?, ?)",
            )
            .bind(uid).bind(&item.id).bind(qty).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            sqlx::query(
                "INSERT OR IGNORE INTO user_item_owned (account_id, item_id) VALUES (?, ?)",
            )
            .bind(uid)
            .bind(&item.id)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        // cards: max level exp, 4 limit breaks, 5 blooms
        let level_limit_breaks = 4i32;
        let max_potential = 5i32;
        for card in Card::table() {
            let max_exp = CardLevel::table()
                .iter()
                .filter(|c| c.group_id == card.card_level_group_id)
                .max_by_key(|c| c.level)
                .map(|c| c.exp)
                .unwrap_or(0);
            let potential = CardPotential::table()
                .iter()
                .filter(|p| p.group_id == card.card_potential_group_id)
                .max_by_key(|p| p.upgrade_count)
                .map(|p| p.upgrade_count)
                .unwrap_or(0)
                .max(max_potential)
                .min(max_potential);
            sqlx::query(
                "INSERT INTO user_cards (account_id, card_id, exp, level_limit_break_count, potential_upgrade_count, potential_upgrade_point_quantity, acquired_time) VALUES (?, ?, ?, ?, ?, 0, ?)",
            )
            .bind(uid).bind(&card.id).bind(max_exp).bind(level_limit_breaks).bind(potential).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        let mut first_character = String::new();
        for ch in Character::table() {
            if first_character.is_empty() {
                first_character = ch.id.clone();
            }
            sqlx::query(
                "INSERT INTO user_characters (account_id, character_id, costume_id, sd_costume_id, exp, acquired_time, last_reward_received_level) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(uid).bind(&ch.id).bind(&ch.default_costume_id).bind(&ch.default_sd_costume_id)
            .bind(1_000_000i64).bind(now).bind(50i32)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        // the park screen uses the profile's park character; an empty id makes the client throw CharacterMaster KeyNotFound
        sqlx::query("UPDATE user_profiles SET park_character_id = ? WHERE account_id = ?")
            .bind(&first_character)
            .bind(uid)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        for c in Costume::table() {
            sqlx::query("INSERT OR IGNORE INTO user_costumes (account_id, costume_id, acquired_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&c.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for c in SdCostume::table() {
            sqlx::query("INSERT OR IGNORE INTO user_sd_costumes (account_id, sd_costume_id, acquired_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&c.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for c in SdCostumeHairAccessory::table() {
            sqlx::query("INSERT OR IGNORE INTO user_sd_costume_hair_accessories (account_id, sd_costume_hair_accessory_id, read_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&c.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        let node_groups: Vec<&str> = SkillTreeNode::table()
            .iter()
            .map(|n| n.group_id.as_str())
            .collect();
        let character_ids: Vec<String> = Character::table().iter().map(|c| c.id.clone()).collect();
        for ch in &character_ids {
            for g in &node_groups {
                sqlx::query(
                    "INSERT OR IGNORE INTO user_character_skill_tree_released (account_id, character_id, node_group_id) VALUES (?, ?, ?)",
                )
                .bind(uid).bind(ch).bind(g)
                .execute(&mut **tx).await.map_err(|e| e.to_string())?;
                sqlx::query(
                    "INSERT OR IGNORE INTO user_character_skill_tree_connected (account_id, character_id, node_group_id) VALUES (?, ?, ?)",
                )
                .bind(uid).bind(ch).bind(g)
                .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        for p in SkillTreePoint::table() {
            sqlx::query("INSERT OR IGNORE INTO user_skill_tree_points (account_id, skill_tree_point_id, quantity) VALUES (?, ?, ?)")
            .bind(uid).bind(&p.id).bind(999_999i64)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        let mut progress_by_mission: HashMap<&str, (&MissionProgress, Vec<i64>)> = HashMap::new();
        for mp in MissionProgress::table() {
            let entry = progress_by_mission
                .entry(mp.mission_id.as_str())
                .or_insert((mp, Vec::new()));
            entry.1.push(mp.threshold);
        }
        for mission in Mission::table() {
            let progress = progress_by_mission
                .get(mission.id.as_str())
                .map(|(last, thresholds)| (last.threshold, thresholds.clone()))
                .unwrap_or((1, vec![1]));
            sqlx::query(
                "INSERT OR IGNORE INTO user_missions (account_id, mission_id, progress, last_progress_time) VALUES (?, ?, ?, ?)",
            )
            .bind(uid).bind(&mission.id).bind(progress.0).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            for t in &progress.1 {
                sqlx::query(
                    "INSERT OR IGNORE INTO user_mission_reward_thresholds (account_id, mission_id, threshold) VALUES (?, ?, ?)",
                )
                .bind(uid).bind(&mission.id).bind(t)
                .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }

        for pass in MissionPass::table() {
            let max_level = MissionPassLevel::table()
                .iter()
                .filter(|l| l.group_id == pass.mission_pass_level_group_id)
                .map(|l| l.level)
                .max()
                .unwrap_or(0);
            sqlx::query(
                "INSERT OR IGNORE INTO user_mission_passes (account_id, mission_pass_id, premium_pass_released_time, finished_time) VALUES (?, ?, ?, ?)",
            )
            .bind(uid).bind(&pass.id).bind(now).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            for level in 1..=max_level {
                for is_premium in [false, true] {
                    sqlx::query(
                        "INSERT OR IGNORE INTO user_mission_pass_received_levels (account_id, mission_pass_id, level, is_premium) VALUES (?, ?, ?, ?)",
                    )
                    .bind(uid).bind(&pass.id).bind(level).bind(is_premium as i32)
                    .execute(&mut **tx).await.map_err(|e| e.to_string())?;
                }
            }
            sqlx::query("INSERT OR IGNORE INTO user_mission_pass_points (account_id, mission_pass_point_id, quantity) VALUES (?, ?, ?)")
            .bind(uid).bind(&pass.mission_pass_point_id).bind(999_999i64)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        for m in Membership::table() {
            sqlx::query(
                "INSERT OR IGNORE INTO user_memberships (account_id, shop_charge_item_id, status, purchased_time, expired_time, is_auto_renew, total_join_month_count) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(uid).bind(&m.shop_charge_item_subscription_shop_charge_item_id)
            .bind(1i32).bind(now).bind(now + 30 * 86400).bind(1i32).bind(1i32)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        for read_type in 0..=200 {
            sqlx::query("INSERT OR IGNORE INTO user_notification_read_times (account_id, notification_read_type, read_time) VALUES (?, ?, ?)")
            .bind(uid).bind(read_type).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        {
            let mut max_step: BTreeMap<i32, i32> = BTreeMap::new();
            for t in Tutorial::table() {
                let e = max_step.entry(t.r#type).or_insert(0);
                *e = (*e).max(t.step);
            }
            for (typ, step) in max_step {
                sqlx::query(
                    "INSERT OR IGNORE INTO user_tutorials (account_id, type, step) VALUES (?, ?, ?)",
                )
                .bind(uid)
                .bind(typ)
                .bind(step)
                .execute(&mut **tx)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        for s in Stamp::table() {
            sqlx::query("INSERT OR IGNORE INTO user_stamps (account_id, stamp_id, is_active) VALUES (?, ?, ?)")
            .bind(uid).bind(&s.id).bind(0i32)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for e in Emblem::table() {
            sqlx::query("INSERT OR IGNORE INTO user_emblems (account_id, emblem_id, last_activated_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&e.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for m in Music::table() {
            sqlx::query("INSERT OR IGNORE INTO user_musics (account_id, music_id, released_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&m.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for s in LiveSkin::table() {
            sqlx::query(
                "INSERT OR IGNORE INTO user_live_skins (account_id, live_skin_id) VALUES (?, ?)",
            )
            .bind(uid)
            .bind(&s.id)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }
        for w in Wallpaper::table() {
            sqlx::query(
                "INSERT OR IGNORE INTO user_wallpapers (account_id, wallpaper_id) VALUES (?, ?)",
            )
            .bind(uid)
            .bind(&w.id)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }
        for p in ParkEmotion::table() {
            sqlx::query("INSERT OR IGNORE INTO user_park_emotions (account_id, park_emotion_id, acquired_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&p.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for p in ParkAccessory::table() {
            sqlx::query("INSERT OR IGNORE INTO user_park_accessories (account_id, park_accessory_id) VALUES (?, ?)")
            .bind(uid).bind(&p.id)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for a in AreaSelector::table() {
            sqlx::query("INSERT OR IGNORE INTO user_park_area_selectors (account_id, area_selector_id, selected_area_id) VALUES (?, ?, ?)")
            .bind(uid).bind(&a.id).bind(&a.area_grouping_id)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for s in Story::table() {
            sqlx::query("INSERT OR IGNORE INTO user_stories (account_id, story_id, read_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&s.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for p in Poster::table() {
            sqlx::query("INSERT OR IGNORE INTO user_posters (account_id, poster_id) VALUES (?, ?)")
                .bind(uid)
                .bind(&p.id)
                .execute(&mut **tx)
                .await
                .map_err(|e| e.to_string())?;
        }
        for t in InstantTips::table() {
            sqlx::query("INSERT OR IGNORE INTO user_instant_tips (account_id, instant_tips_id) VALUES (?, ?)")
            .bind(uid).bind(&t.id)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for f in Facility::table() {
            sqlx::query("INSERT OR IGNORE INTO user_facilities (account_id, facility_id, current_level) VALUES (?, ?, ?)")
            .bind(uid).bind(&f.id).bind(1i32)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }
        for f in FanMark::table() {
            sqlx::query("INSERT OR IGNORE INTO user_fan_marks (account_id, fan_mark_id, acquired_time) VALUES (?, ?, ?)")
            .bind(uid).bind(&f.id).bind(now)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
        }

        let mut last_cleared_main = String::new();
        let mut last_cleared_main_step = 0i32;
        for q in ParkQuest::table() {
            sqlx::query(
                "INSERT OR IGNORE INTO user_park_quests (account_id, park_quest_id, start_time, is_progress, clear_time, clear_count) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(uid).bind(&q.id).bind(now).bind(0i32).bind(now).bind(1i64)
            .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            let is_main = q.park_quest_chapter_id.starts_with("quest_chapter-main");
            if is_main {
                let mut max_step = 0i32;
                for st in ParkQuestStep::table()
                    .iter()
                    .filter(|s| s.park_quest_id == q.id)
                {
                    max_step = max_step.max(st.step);
                }
                // main quest ids are zero-padded and sortable as strings; keep the last quest of the chain
                if q.id.as_str() > last_cleared_main.as_str() {
                    last_cleared_main = q.id.clone();
                    last_cleared_main_step = max_step;
                }
            }
            let mut groups: BTreeMap<i32, i32> = BTreeMap::new();
            for s in ParkQuestStep::table()
                .iter()
                .filter(|s| s.park_quest_id == q.id)
            {
                let e = groups.entry(s.step_group_number).or_insert(0);
                *e = (*e).max(s.step);
            }
            for (group, step) in groups {
                sqlx::query(
                    "INSERT OR IGNORE INTO user_park_quest_steps (account_id, park_quest_id, step_group_number, current_step, all_clear_time) VALUES (?, ?, ?, ?, ?)",
                )
                .bind(uid).bind(&q.id).bind(group).bind(step).bind(now)
                .execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }

        for level in 1..=max_level_row.level {
            sqlx::query(
                "INSERT OR IGNORE INTO user_park_level_rewards (account_id, level) VALUES (?, ?)",
            )
            .bind(uid)
            .bind(level)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;
        }
        let first_char = character_ids.first().cloned().unwrap_or_default();
        sqlx::query(
            "INSERT INTO user_parks (account_id, time_point, current_area_id, initial_park_character_id, last_cleared_main_park_quest_id, last_cleared_main_park_quest_step) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(uid)
        .bind(14i32)
        .bind("area-a01")
        .bind(first_char)
        .bind(&last_cleared_main)
        .bind(last_cleared_main_step as i64)
        .execute(&mut **tx).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;

    #[tokio::test]
    async fn create_and_max_account() {
        let dir = std::env::temp_dir().join(format!("symtest-{}", std::process::id()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let db_path = dir.join("test.db");
        let db = Database::open(&db_path).await.expect("open db");

        let uid = db
            .get_or_create_account("test-credential-1")
            .await
            .expect("create");
        // idempotent: same credential returns same account
        let uid2 = db
            .get_or_create_account("test-credential-1")
            .await
            .expect("again");
        assert_eq!(uid, uid2);

        let data = db.user_data(&uid).await.expect("snapshot");
        assert!(data.user_card_list.len() > 160);
        assert_eq!(
            data.user_balance_free.as_ref().unwrap().free_quantity,
            999_999
        );
        assert_eq!(
            data.user_balance_paid.as_ref().unwrap().paid_quantity,
            999_999
        );
        assert!(data.user_mission_list.len() >= 1500);
        assert!(data.user_character_list.len() >= 50);
        assert!(data.user_shop_charge_item_subscription_list.len() >= 50);
        assert!(data.user_music_list.len() > 100);
        // the park screen resolves the current character from the profile; must be set
        assert!(
            !data
                .user_profile
                .as_ref()
                .unwrap()
                .park_character_id
                .is_empty(),
            "park_character_id must be set (client throws CharacterMaster KeyNotFound otherwise)"
        );
        // tutorials: one row per type (no duplicate types), GAME_START done
        let mut seen = std::collections::HashSet::new();
        for t in &data.user_tutorial_list {
            assert!(
                seen.insert(t.r#type),
                "duplicate tutorial type {}",
                t.r#type
            );
        }
        assert!(
            data.user_tutorial_list
                .iter()
                .any(|t| t.r#type == 1 && t.step >= 10),
            "GAME_START tutorial missing"
        );
        // the client pairs connected skill-tree cards with groups by index; misaligned lists throw
        for st in &data.user_character_skill_tree_list {
            assert_eq!(
                st.connected_skill_tree_node_group_ids.len(),
                st.connected_skill_tree_node_card_ids.len(),
                "character {}: connected groups/cards must align",
                st.character_id
            );
            assert!(
                st.connected_skill_tree_node_card_ids
                    .iter()
                    .all(|c| !c.is_empty()),
                "character {}: connected card ids must be non-empty",
                st.character_id
            );
        }
        // park quests all cleared, last main quest recorded
        assert!(data.user_park_quest_list.len() >= 200);
        assert!(
            !data
                .user_park
                .as_ref()
                .unwrap()
                .last_cleared_main_park_quest_id
                .is_empty()
        );
        // every card maxed: 4 limit breaks, 5 blooms, positive exp
        for card in &data.user_card_list {
            assert_eq!(card.level_limit_break_count, 4, "{}", card.card_id);
            assert_eq!(card.potential_upgrade_count, 5, "{}", card.card_id);
            assert!(card.exp > 0, "{}", card.card_id);
        }

        // inventory ops
        let (free, paid) = db.balances(&uid).await.unwrap();
        assert_eq!((free, paid), (999_999, 999_999));
        assert!(db.consume_stones(&uid, 2500, 0).await.unwrap());
        let (free, _) = db.balances(&uid).await.unwrap();
        assert_eq!(free, 999_999 - 2500);
        assert!(!db.consume_stones(&uid, 10_000_000, 0).await.unwrap());

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}

impl Database {
    /// Record a tutorial step (upsert: never lowers the step).
    pub async fn set_tutorial_step(
        &self,
        uid: &str,
        tutorial_type: i32,
        step: i32,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_tutorials (account_id, type, step) VALUES (?, ?, ?)
            ON CONFLICT(account_id, type) DO UPDATE SET step = MAX(step, excluded.step)",
        )
        .bind(uid)
        .bind(tutorial_type)
        .bind(step)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// Store the country / birthday / name chosen during onboarding.
    pub async fn set_initial_user_info(
        &self,
        uid: &str,
        country_code: &str,
        birth_year: i32,
        birth_month: i32,
        name: &str,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users SET country_code = ?, birth_year_for_payment = ?, birth_month_for_payment = ? WHERE account_id = ?",
        )
        .bind(country_code)
        .bind(birth_year)
        .bind(birth_month)
        .bind(uid)
        .execute(self.pool())
        .await?;
        sqlx::query("UPDATE user_profiles SET name = ? WHERE account_id = ?")
            .bind(name)
            .bind(uid)
            .execute(self.pool())
            .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// Set the park character chosen in onboarding.
    pub async fn set_park_character(&self, uid: &str, character_id: &str) -> Result<(), Error> {
        sqlx::query("UPDATE user_profiles SET park_character_id = ? WHERE account_id = ?")
            .bind(character_id)
            .bind(uid)
            .execute(self.pool())
            .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// whether the account owns a character (park member selection).
    pub async fn owns_character(&self, uid: &str, character_id: &str) -> Result<bool, Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT character_id FROM user_characters WHERE account_id = ? AND character_id = ?",
        )
        .bind(uid)
        .bind(character_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.is_some())
    }

    /// whether the account owns a costume.
    pub async fn owns_costume(&self, uid: &str, costume_id: &str) -> Result<bool, Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT costume_id FROM user_costumes WHERE account_id = ? AND costume_id = ?",
        )
        .bind(uid)
        .bind(costume_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.is_some())
    }

    /// whether the account owns an sd costume.
    pub async fn owns_sd_costume(&self, uid: &str, sd_costume_id: &str) -> Result<bool, Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT sd_costume_id FROM user_sd_costumes WHERE account_id = ? AND sd_costume_id = ?",
        )
        .bind(uid)
        .bind(sd_costume_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.is_some())
    }

    /// the character's equipped costume ids: (costume_id, sd_costume_id, sd_costume_hair_accessory_id).
    pub async fn character_costume_ids(
        &self,
        uid: &str,
        character_id: &str,
    ) -> Result<Option<(String, String, String)>, Error> {
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT costume_id, sd_costume_id, sd_costume_hair_accessory_id FROM user_characters WHERE account_id = ? AND character_id = ?",
        )
        .bind(uid)
        .bind(character_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row)
    }

    /// equip a character's live costume.
    pub async fn set_character_costume(
        &self,
        uid: &str,
        character_id: &str,
        costume_id: &str,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_characters SET costume_id = ? WHERE account_id = ? AND character_id = ?",
        )
        .bind(costume_id)
        .bind(uid)
        .bind(character_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// equip a character's sd costume + hair accessory.
    pub async fn set_character_sd_costume(
        &self,
        uid: &str,
        character_id: &str,
        sd_costume_id: &str,
        sd_costume_hair_accessory_id: &str,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_characters SET sd_costume_id = ?, sd_costume_hair_accessory_id = ? WHERE account_id = ? AND character_id = ?",
        )
        .bind(sd_costume_id)
        .bind(sd_costume_hair_accessory_id)
        .bind(uid)
        .bind(character_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// mark a costume's detail page as read.
    pub async fn mark_costume_read(
        &self,
        uid: &str,
        costume_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_costumes SET read_time = ? WHERE account_id = ? AND costume_id = ?",
        )
        .bind(now)
        .bind(uid)
        .bind(costume_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// mark an sd costume's detail page as read.
    pub async fn mark_sd_costume_read(
        &self,
        uid: &str,
        sd_costume_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_sd_costumes SET read_time = ? WHERE account_id = ? AND sd_costume_id = ?",
        )
        .bind(now)
        .bind(uid)
        .bind(sd_costume_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// mark an sd costume hair accessory as read.
    pub async fn mark_sd_costume_hair_accessory_read(
        &self,
        uid: &str,
        hair_accessory_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_sd_costume_hair_accessories SET read_time = ? WHERE account_id = ? AND sd_costume_hair_accessory_id = ?",
        )
        .bind(now)
        .bind(uid)
        .bind(hair_accessory_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// mark a character's detail page as read.
    pub async fn mark_character_read(
        &self,
        uid: &str,
        character_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_characters SET read_time = ? WHERE account_id = ? AND character_id = ?",
        )
        .bind(now)
        .bind(uid)
        .bind(character_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// mark a character's park data as read.
    pub async fn mark_character_park_read(
        &self,
        uid: &str,
        character_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_characters SET park_read_time = ? WHERE account_id = ? AND character_id = ?",
        )
        .bind(now)
        .bind(uid)
        .bind(character_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// player levels whose rewards were received (park level rewards).
    pub async fn park_received_levels(&self, uid: &str) -> Result<Vec<i32>, Error> {
        let rows: Vec<i32> = sqlx::query_scalar(
            "SELECT level FROM user_park_level_rewards WHERE account_id = ? ORDER BY level",
        )
        .bind(uid)
        .fetch_all(self.pool())
        .await?;
        Ok(rows)
    }

    /// record a received player level reward.
    pub async fn add_park_level_reward(&self, uid: &str, level: i32) -> Result<(), Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO user_park_level_rewards (account_id, level) VALUES (?, ?)",
        )
        .bind(uid)
        .bind(level)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// a user's card row (exp / limit breaks / potential upgrades).
    pub async fn user_card(&self, uid: &str, card_id: &str) -> Result<Option<UserCardRow>, Error> {
        sqlx::query_as::<_, UserCardRow>(
            "SELECT * FROM user_cards WHERE account_id = ? AND card_id = ?",
        )
        .bind(uid)
        .bind(card_id)
        .fetch_optional(self.pool())
        .await
    }

    /// all of a user's card rows (exp / limit breaks / potential upgrades).
    pub async fn user_cards(&self, uid: &str) -> Result<Vec<UserCardRow>, Error> {
        sqlx::query_as::<_, UserCardRow>("SELECT * FROM user_cards WHERE account_id = ?")
            .bind(uid)
            .fetch_all(self.pool())
            .await
    }

    /// node group ids released in a character's skill tree.
    pub async fn skill_tree_released_groups(
        &self,
        uid: &str,
        character_id: &str,
    ) -> Result<Vec<String>, Error> {
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT node_group_id FROM user_character_skill_tree_released WHERE account_id = ? AND character_id = ?",
        )
        .bind(uid)
        .bind(character_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows)
    }

    /// Mark an instant tip as read.
    pub async fn read_instant_tip(&self, uid: &str, tip_id: &str) -> Result<(), Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO user_instant_tips (account_id, instant_tips_id) VALUES (?, ?)",
        )
        .bind(uid)
        .bind(tip_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// when a notice was last read (None = never).
    pub async fn notice_read_time(&self, uid: &str, notice_id: &str) -> Result<Option<i64>, Error> {
        let t: Option<i64> = sqlx::query_scalar(
            "SELECT read_time FROM user_notice_read_times WHERE account_id = ? AND notice_id = ?",
        )
        .bind(uid)
        .bind(notice_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(t)
    }

    /// record a notice read time (replace).
    pub async fn set_notice_read_time(
        &self,
        uid: &str,
        notice_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO user_notice_read_times (account_id, notice_id, read_time) VALUES (?, ?, ?)",
        )
        .bind(uid)
        .bind(notice_id)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// release skill-tree node groups for a character (idempotent).
    pub async fn release_skill_tree_node(
        &self,
        uid: &str,
        character_id: &str,
        groups: &[String],
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        for group in groups {
            sqlx::query(
                "INSERT OR IGNORE INTO user_character_skill_tree_released (account_id, character_id, node_group_id) VALUES (?, ?, ?)",
            )
            .bind(uid)
            .bind(character_id)
            .bind(group)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// connect a released node group to a card (idempotent).
    pub async fn connect_skill_tree_node(
        &self,
        uid: &str,
        character_id: &str,
        node_group_id: &str,
        card_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query(
            "INSERT OR IGNORE INTO user_character_skill_tree_connected (account_id, character_id, node_group_id) VALUES (?, ?, ?)",
        )
        .bind(uid)
        .bind(character_id)
        .bind(node_group_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT OR IGNORE INTO user_character_skill_tree_connected_cards (account_id, character_id, card_id) VALUES (?, ?, ?)",
        )
        .bind(uid)
        .bind(character_id)
        .bind(card_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// reset (delete) released/connected state for the given node groups.
    pub async fn reset_skill_tree_node(
        &self,
        uid: &str,
        character_id: &str,
        groups: &[String],
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        for group in groups {
            sqlx::query(
                "DELETE FROM user_character_skill_tree_released WHERE account_id = ? AND character_id = ? AND node_group_id = ?",
            )
            .bind(uid)
            .bind(character_id)
            .bind(group)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "DELETE FROM user_character_skill_tree_connected WHERE account_id = ? AND character_id = ? AND node_group_id = ?",
            )
            .bind(uid)
            .bind(character_id)
            .bind(group)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// set the profile display name (+ the client-facing update timestamp).
    pub async fn set_profile_name(&self, uid: &str, name: &str, now: i64) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_profiles SET name = ?, name_last_updated_time = ? WHERE account_id = ?",
        )
        .bind(name)
        .bind(now)
        .bind(uid)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// set the profile message (+ the client-facing update timestamp).
    pub async fn set_profile_message(
        &self,
        uid: &str,
        message: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query("UPDATE user_profiles SET message = ?, message_last_updated_time = ? WHERE account_id = ?")
            .bind(message)
            .bind(now)
            .bind(uid)
            .execute(self.pool())
            .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// set the profile fan mark.
    pub async fn set_profile_fan_mark(&self, uid: &str, fan_mark_id: &str) -> Result<(), Error> {
        sqlx::query("UPDATE user_profiles SET fan_mark_id = ? WHERE account_id = ?")
            .bind(fan_mark_id)
            .bind(uid)
            .execute(self.pool())
            .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// replace the profile's emblem positions wholesale.
    pub async fn set_profile_emblem_positions(
        &self,
        uid: &str,
        positions: &[(i32, String)],
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query("DELETE FROM user_profile_emblem_positions WHERE account_id = ?")
            .bind(uid)
            .execute(&mut *tx)
            .await?;
        for (pos, emblem_id) in positions {
            sqlx::query(
                "INSERT INTO user_profile_emblem_positions (account_id, position, emblem_id) VALUES (?, ?, ?)",
            )
            .bind(uid)
            .bind(pos)
            .bind(emblem_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// save (or replace) a custom palette's image URL + background card.
    pub async fn set_custom_palette(
        &self,
        uid: &str,
        number: i32,
        image_url: &str,
        background_card_id: &str,
        background_potential: i32,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_custom_palettes (account_id, number, image_url, is_inactivated, background_card_id, background_card_potential_upgrade_count)
             VALUES (?, ?, ?, 0, ?, ?)
             ON CONFLICT(account_id, number) DO UPDATE SET
                 image_url = excluded.image_url, is_inactivated = 0,
                 background_card_id = excluded.background_card_id,
                 background_card_potential_upgrade_count = excluded.background_card_potential_upgrade_count",
        )
        .bind(uid)
        .bind(number)
        .bind(image_url)
        .bind(background_card_id)
        .bind(background_potential)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// make a palette active; the client displays the palette whose number equals the profile's.
    pub async fn set_profile_custom_palette(&self, uid: &str, number: i32) -> Result<(), Error> {
        sqlx::query("UPDATE user_profiles SET custom_palette_number = ? WHERE account_id = ?")
            .bind(number)
            .bind(uid)
            .execute(self.pool())
            .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// all saved custom palettes for the profile palette list.
    pub async fn custom_palettes(
        &self,
        uid: &str,
    ) -> Result<Vec<crate::models::UserCustomPaletteRow>, Error> {
        sqlx::query_as::<_, crate::models::UserCustomPaletteRow>(
            "SELECT * FROM user_custom_palettes WHERE account_id = ? ORDER BY number",
        )
        .bind(uid)
        .fetch_all(self.pool())
        .await
    }

    /// replace a palette's parts (delete + insert).
    pub async fn set_custom_palette_parts(
        &self,
        uid: &str,
        number: i32,
        parts: &[types::rpc::api::common::CustomPalettePart],
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query("DELETE FROM user_custom_palette_parts WHERE account_id = ? AND number = ?")
            .bind(uid)
            .bind(number)
            .execute(&mut *tx)
            .await?;
        for (i, p) in parts.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_custom_palette_parts
                 (account_id, number, part_index, resource_type, resource_id, position_x_permil, position_y_permil, scale_permil, rotation_permil, layer)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(uid)
            .bind(number)
            .bind(i as i32)
            .bind(p.resource_type)
            .bind(&p.resource_id)
            .bind(p.position_x_permil)
            .bind(p.position_y_permil)
            .bind(p.scale_permil)
            .bind(p.rotation_permil)
            .bind(p.layer)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// a palette's parts, in insertion order.
    pub async fn custom_palette_parts(
        &self,
        uid: &str,
        number: i32,
    ) -> Result<Vec<crate::models::UserCustomPalettePartRow>, Error> {
        sqlx::query_as::<_, crate::models::UserCustomPalettePartRow>(
            "SELECT * FROM user_custom_palette_parts WHERE account_id = ? AND number = ? ORDER BY part_index",
        )
        .bind(uid)
        .bind(number)
        .fetch_all(self.pool())
        .await
    }
}
