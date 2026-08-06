use sqlx::Error;

use crate::Database;
use crate::models::{UserJumpRopeNpcExitRow, UserJumpRopeRow};

impl Database {
    /// a user's jump-rope row (best count / cleared / play count).
    pub async fn jump_rope(
        &self,
        uid: &str,
        jump_rope_id: &str,
    ) -> Result<Option<UserJumpRopeRow>, Error> {
        sqlx::query_as::<_, UserJumpRopeRow>(
            "SELECT * FROM user_jump_ropes WHERE account_id = ? AND jump_rope_id = ?",
        )
        .bind(uid)
        .bind(jump_rope_id)
        .fetch_optional(self.pool())
        .await
    }

    /// the NPC exit jump counts for a jump-rope attempt, in exit order.
    pub async fn jump_rope_npc_exits(
        &self,
        uid: &str,
        jump_rope_id: &str,
    ) -> Result<Vec<UserJumpRopeNpcExitRow>, Error> {
        sqlx::query_as::<_, UserJumpRopeNpcExitRow>(
            "SELECT * FROM user_jump_rope_npc_exits WHERE account_id = ? AND jump_rope_id = ? ORDER BY exit_index",
        )
        .bind(uid)
        .bind(jump_rope_id)
        .fetch_all(self.pool())
        .await
    }

    /// record a started single, preserving best and cleared state.
    pub async fn start_jump_rope(
        &self,
        uid: &str,
        jump_rope_id: &str,
        npc_exits: &[i32],
        now: i64,
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query(
            "INSERT INTO user_jump_ropes (account_id, jump_rope_id, best_jump_count, is_cleared, play_count, notification_read_time, last_started_time)
             VALUES (?, ?, 0, 0, 1, ?, ?)
             ON CONFLICT(account_id, jump_rope_id) DO UPDATE SET play_count = play_count + 1, last_started_time = excluded.last_started_time",
        )
        .bind(uid)
        .bind(jump_rope_id)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM user_jump_rope_npc_exits WHERE account_id = ? AND jump_rope_id = ?",
        )
        .bind(uid)
        .bind(jump_rope_id)
        .execute(&mut *tx)
        .await?;
        for (i, count) in npc_exits.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_jump_rope_npc_exits (account_id, jump_rope_id, exit_index, jump_count) VALUES (?, ?, ?, ?)",
            )
            .bind(uid)
            .bind(jump_rope_id)
            .bind(i as i32)
            .bind(count)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }

    /// the in-flight rope: the user's most recently started jump rope.
    pub async fn last_started_jump_rope(
        &self,
        uid: &str,
    ) -> Result<Option<UserJumpRopeRow>, Error> {
        sqlx::query_as::<_, UserJumpRopeRow>(
            "SELECT * FROM user_jump_ropes WHERE account_id = ? ORDER BY last_started_time DESC, rowid DESC LIMIT 1",
        )
        .bind(uid)
        .fetch_optional(self.pool())
        .await
    }

    /// record a finished single, keeping the best count and cleared flag.
    pub async fn finish_jump_rope(
        &self,
        uid: &str,
        jump_rope_id: &str,
        best: i64,
        cleared: bool,
        now: i64,
    ) -> Result<(), Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query(
            "INSERT INTO user_jump_ropes (account_id, jump_rope_id, best_jump_count, is_cleared, play_count, notification_read_time)
             VALUES (?, ?, ?, ?, 0, ?)
             ON CONFLICT(account_id, jump_rope_id) DO UPDATE SET
                 best_jump_count = MAX(best_jump_count, excluded.best_jump_count),
                 is_cleared = MAX(is_cleared, excluded.is_cleared)",
        )
        .bind(uid)
        .bind(jump_rope_id)
        .bind(best)
        .bind(cleared)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM user_jump_rope_npc_exits WHERE account_id = ? AND jump_rope_id = ?",
        )
        .bind(uid)
        .bind(jump_rope_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.invalidate(uid);
        Ok(())
    }
}
