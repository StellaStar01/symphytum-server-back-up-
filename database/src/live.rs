use super::Database;
use crate::models::{UserLiveDeckPositionRow, UserLiveDeckRow, UserLiveRow};
use sqlx::Error;

/// a saved deck with its 1..=5 positions.
pub struct Deck {
    pub character_id: String,
    pub number: i32,
    pub name: String,
    pub costume_id: String,
    pub positions: Vec<(i32, String)>, // (position, card_id)
}

impl Database {
    pub async fn get_deck(
        &self,
        uid: &str,
        character_id: &str,
        number: i32,
    ) -> Result<Option<Deck>, Error> {
        let deck: Option<UserLiveDeckRow> = sqlx::query_as(
            "SELECT * FROM user_live_decks WHERE account_id = ? AND character_id = ? AND number = ?",
        )
        .bind(uid)
        .bind(character_id)
        .bind(number)
        .fetch_optional(self.pool())
        .await?;
        let Some(deck) = deck else { return Ok(None) };
        let positions: Vec<UserLiveDeckPositionRow> = sqlx::query_as(
            "SELECT * FROM user_live_deck_positions WHERE account_id = ? AND character_id = ? AND number = ? ORDER BY position",
        )
        .bind(uid)
        .bind(character_id)
        .bind(number)
        .fetch_all(self.pool())
        .await?;
        Ok(Some(Deck {
            character_id: deck.character_id,
            number: deck.number,
            name: deck.name,
            costume_id: deck.costume_id,
            positions: positions
                .into_iter()
                .map(|p| (p.position, p.card_id))
                .collect(),
        }))
    }

    pub async fn save_deck(
        &self,
        uid: &str,
        character_id: &str,
        number: i32,
        name: &str,
        costume_id: &str,
        positions: &[(i32, String)],
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query(
            "INSERT INTO user_live_decks (account_id, character_id, number, name, costume_id) VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(account_id, character_id, number) DO UPDATE SET name = excluded.name, costume_id = excluded.costume_id",
        )
        .bind(uid)
        .bind(character_id)
        .bind(number)
        .bind(name)
        .bind(costume_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM user_live_deck_positions WHERE account_id = ? AND character_id = ? AND number = ?",
        )
        .bind(uid)
        .bind(character_id)
        .bind(number)
        .execute(&mut *tx)
        .await?;
        for (position, card_id) in positions {
            sqlx::query(
                "INSERT INTO user_live_deck_positions (account_id, character_id, number, position, card_id) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(uid)
            .bind(character_id)
            .bind(number)
            .bind(position)
            .bind(card_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    pub async fn get_live(&self, uid: &str) -> Result<UserLiveRow, Error> {
        let row = sqlx::query_as::<_, UserLiveRow>("SELECT * FROM user_lives WHERE account_id = ?")
            .bind(uid)
            .fetch_optional(self.pool())
            .await?;
        Ok(row.unwrap_or(UserLiveRow {
            account_id: uid.to_string(),
            reward_up_stamina: 0,
            last_reward_up_stamina_auto_recovery_time: 0,
            reward_up_stamina_consumption_setting_quantity: 1,
            last_single_played_music_id: String::new(),
            last_single_played_music_difficulty_type: 0,
            last_multi_selected_music_id: String::new(),
            last_multi_selected_music_difficulty_type: 0,
            last_watched_live_deck_character_id: String::new(),
            last_watched_live_deck_number: 0,
            last_played_character_id: String::new(),
        }))
    }

    pub async fn save_live(&self, live: &UserLiveRow) -> Result<(), Error> {
        let uid = &live.account_id;
        sqlx::query(
            "INSERT INTO user_lives (account_id, reward_up_stamina, last_reward_up_stamina_auto_recovery_time, reward_up_stamina_consumption_setting_quantity, last_single_played_music_id, last_single_played_music_difficulty_type, last_multi_selected_music_id, last_multi_selected_music_difficulty_type, last_watched_live_deck_character_id, last_watched_live_deck_number, last_played_character_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id) DO UPDATE SET
               reward_up_stamina = excluded.reward_up_stamina,
               last_reward_up_stamina_auto_recovery_time = excluded.last_reward_up_stamina_auto_recovery_time,
               reward_up_stamina_consumption_setting_quantity = excluded.reward_up_stamina_consumption_setting_quantity,
               last_single_played_music_id = excluded.last_single_played_music_id,
               last_single_played_music_difficulty_type = excluded.last_single_played_music_difficulty_type,
               last_multi_selected_music_id = excluded.last_multi_selected_music_id,
               last_multi_selected_music_difficulty_type = excluded.last_multi_selected_music_difficulty_type,
               last_watched_live_deck_character_id = excluded.last_watched_live_deck_character_id,
               last_watched_live_deck_number = excluded.last_watched_live_deck_number,
               last_played_character_id = excluded.last_played_character_id",
        )
        .bind(uid)
        .bind(live.reward_up_stamina)
        .bind(live.last_reward_up_stamina_auto_recovery_time)
        .bind(live.reward_up_stamina_consumption_setting_quantity)
        .bind(&live.last_single_played_music_id)
        .bind(live.last_single_played_music_difficulty_type)
        .bind(&live.last_multi_selected_music_id)
        .bind(live.last_multi_selected_music_difficulty_type)
        .bind(&live.last_watched_live_deck_character_id)
        .bind(live.last_watched_live_deck_number)
        .bind(&live.last_played_character_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// record a finished single live; only the best scores and clear counts update.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_single_result(
        &self,
        uid: &str,
        music_id: &str,
        difficulty_type: i32,
        character_id: &str,
        costume_id: &str,
        score: i64,
        max_combo: i64,
        clear: bool,
        live_result_type: i32,
    ) -> Result<(), Error> {
        let now = crate::unix_now();
        let mut tx = self.pool().begin().await?;

        // per-music summary (best score wins)
        let prev_music: Option<(i64,)> = sqlx::query_as(
            "SELECT highest_score FROM user_musics WHERE account_id = ? AND music_id = ?",
        )
        .bind(uid)
        .bind(music_id)
        .fetch_optional(&mut *tx)
        .await?;
        let is_best = prev_music.map(|(s,)| score > s).unwrap_or(true);
        if is_best {
            sqlx::query(
                "INSERT INTO user_musics (account_id, music_id, released_time, highest_score, highest_score_last_updated_time, highest_score_music_difficulty_type, highest_score_character_id, highest_score_costume_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(account_id, music_id) DO UPDATE SET
                   highest_score = excluded.highest_score,
                   highest_score_last_updated_time = excluded.highest_score_last_updated_time,
                   highest_score_music_difficulty_type = excluded.highest_score_music_difficulty_type,
                   highest_score_character_id = excluded.highest_score_character_id,
                   highest_score_costume_id = excluded.highest_score_costume_id",
            )
            .bind(uid)
            .bind(music_id)
            .bind(now)
            .bind(score)
            .bind(now)
            .bind(difficulty_type)
            .bind(character_id)
            .bind(costume_id)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                "INSERT OR IGNORE INTO user_musics (account_id, music_id, released_time) VALUES (?, ?, ?)",
            )
            .bind(uid)
            .bind(music_id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }

        // per-difficulty: best score + clear count
        let prev_diff: Option<(i64, i64)> = sqlx::query_as(
            "SELECT highest_score, clear_count FROM user_music_difficulties WHERE account_id = ? AND music_id = ? AND difficulty_type = ?",
        )
        .bind(uid)
        .bind(music_id)
        .bind(difficulty_type)
        .fetch_optional(&mut *tx)
        .await?;
        let (old_score, old_clear) = prev_diff.unwrap_or((0, 0));
        let new_score = old_score.max(score);
        let new_clear = old_clear + if clear { 1 } else { 0 };
        let new_max_combo = {
            let old_combo: Option<(i64,)> = sqlx::query_as(
                "SELECT max_combo_count FROM user_music_difficulties WHERE account_id = ? AND music_id = ? AND difficulty_type = ?",
            )
            .bind(uid).bind(music_id).bind(difficulty_type)
            .fetch_optional(&mut *tx).await?;
            old_combo.map(|(c,)| c).unwrap_or(0).max(max_combo)
        };
        sqlx::query(
            "INSERT INTO user_music_difficulties (account_id, music_id, difficulty_type, highest_score, max_combo_count, clear_count, live_result_type)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id, music_id, difficulty_type) DO UPDATE SET
               highest_score = excluded.highest_score,
               max_combo_count = excluded.max_combo_count,
               clear_count = excluded.clear_count,
               live_result_type = excluded.live_result_type",
        )
        .bind(uid)
        .bind(music_id)
        .bind(difficulty_type)
        .bind(new_score)
        .bind(new_max_combo)
        .bind(new_clear)
        .bind(live_result_type)
        .execute(&mut *tx)
        .await?;

        // per-character-per-music score (best only)
        if !character_id.is_empty() {
            let prev_char: Option<(i64,)> = sqlx::query_as(
                "SELECT highest_score FROM user_music_character_highest_scores WHERE account_id = ? AND character_id = ? AND music_id = ? AND difficulty_type = ?",
            )
            .bind(uid).bind(character_id).bind(music_id).bind(difficulty_type)
            .fetch_optional(&mut *tx).await?;
            if prev_char.map(|(s,)| score > s).unwrap_or(true) {
                sqlx::query(
                    "INSERT INTO user_music_character_highest_scores (account_id, character_id, music_id, difficulty_type, highest_score, highest_score_last_updated_time)
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON CONFLICT(account_id, character_id, music_id, difficulty_type) DO UPDATE SET
                       highest_score = excluded.highest_score,
                       highest_score_last_updated_time = excluded.highest_score_last_updated_time",
                )
                .bind(uid).bind(character_id).bind(music_id).bind(difficulty_type).bind(score).bind(now)
                .execute(&mut *tx).await?;
            }
        }

        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// best previous per-character score for a music/difficulty (before a new run).
    pub async fn past_character_highest_score(
        &self,
        uid: &str,
        music_id: &str,
        difficulty_type: i32,
    ) -> Result<i64, Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT highest_score FROM user_music_character_highest_scores WHERE account_id = ? AND music_id = ? AND difficulty_type = ?",
        )
        .bind(uid)
        .bind(music_id)
        .bind(difficulty_type)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(|(s,)| s).unwrap_or(0))
    }
}
