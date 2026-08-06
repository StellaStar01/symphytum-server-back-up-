use super::Database;
use crate::unix_now;
use sqlx::Error;

impl Database {
    /// (free, paid) stone balance.
    pub async fn balances(&self, uid: &str) -> Result<(i32, i32), Error> {
        let row: Option<(i32, i32)> = sqlx::query_as(
            "SELECT free_quantity, paid_quantity FROM user_balances WHERE account_id = ?",
        )
        .bind(uid)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.unwrap_or((0, 0)))
    }

    /// grant stones (free/paid delta upsert).
    pub async fn grant_stones(&self, uid: &str, free: i64, paid: i64) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_balances (account_id, free_quantity, paid_quantity)
             VALUES (?, ?, ?)
             ON CONFLICT(account_id) DO UPDATE SET
               free_quantity = free_quantity + excluded.free_quantity,
               paid_quantity = paid_quantity + excluded.paid_quantity",
        )
        .bind(uid)
        .bind(free)
        .bind(paid)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    pub async fn consume_stones(&self, uid: &str, free: i64, paid: i64) -> Result<bool, Error> {
        let (cur_free, cur_paid) = self.balances(uid).await?;
        if (cur_free as i64) + (cur_paid as i64) < free + paid {
            return Ok(false);
        }
        // free (red) stones are spent first; a free-balance deficit spills into paid.
        let used_free = free.min(cur_free as i64);
        let mut used_paid = paid;
        if used_free < free {
            used_paid += free - used_free;
        }
        let new_paid = cur_paid - used_paid as i32;
        let new_free = cur_free - used_free as i32;
        sqlx::query(
            "UPDATE user_balances SET free_quantity = ?, paid_quantity = ? WHERE account_id = ?",
        )
        .bind(new_free)
        .bind(new_paid)
        .bind(uid)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(true)
    }

    /// item quantity for (item_id, expired_time = 0).
    pub async fn item_quantity(&self, uid: &str, item_id: &str) -> Result<i64, Error> {
        let qty: Option<i64> = sqlx::query_scalar(
            "SELECT quantity FROM user_items WHERE account_id = ? AND item_id = ? AND expired_time = 0",
        )
        .bind(uid)
        .bind(item_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(qty.unwrap_or(0))
    }

    /// grant item quantity (upsert), also marking it owned.
    pub async fn add_item(
        &self,
        uid: &str,
        item_id: &str,
        quantity: i64,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_items (account_id, item_id, expired_time, quantity, last_acquired_time) VALUES (?, ?, 0, ?, ?)
             ON CONFLICT(account_id, item_id, expired_time) DO UPDATE SET quantity = quantity + excluded.quantity",
        )
        .bind(uid)
        .bind(item_id)
        .bind(quantity)
        .bind(now)
        .execute(self.pool())
        .await?;
        sqlx::query("INSERT OR IGNORE INTO user_item_owned (account_id, item_id) VALUES (?, ?)")
            .bind(uid)
            .bind(item_id)
            .execute(self.pool())
            .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// consume item quantity; returns false when insufficient
    pub async fn consume_item(
        &self,
        uid: &str,
        item_id: &str,
        quantity: i64,
    ) -> Result<bool, Error> {
        let cur = self.item_quantity(uid, item_id).await?;
        if cur < quantity {
            return Ok(false);
        }
        let new = cur - quantity;
        sqlx::query(
            "UPDATE user_items SET quantity = ? WHERE account_id = ? AND item_id = ? AND expired_time = 0",
        )
        .bind(new)
        .bind(uid)
        .bind(item_id)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(true)
    }

    /// grant a card. returns (is_new, existing_potential_upgrade_count).
    pub async fn grant_card(
        &self,
        uid: &str,
        card_id: &str,
        now: i64,
    ) -> Result<(bool, i32), Error> {
        let existing: Option<(i32, i32)> = sqlx::query_as(
            "SELECT potential_upgrade_count, potential_upgrade_point_quantity FROM user_cards WHERE account_id = ? AND card_id = ?",
        )
        .bind(uid)
        .bind(card_id)
        .fetch_optional(self.pool())
        .await?;
        if let Some((potential, _points)) = existing {
            self.invalidate(uid);
            return Ok((false, potential));
        }
        sqlx::query(
            "INSERT INTO user_cards (account_id, card_id, exp, level_limit_break_count, potential_upgrade_count, potential_upgrade_point_quantity, acquired_time) VALUES (?, ?, 0, 0, 0, 0, ?)",
        )
        .bind(uid)
        .bind(card_id)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok((true, 0))
    }

    /// grant a costume / sd costume (ignore if already owned)
    pub async fn grant_costume(&self, uid: &str, costume_id: &str, now: i64) -> Result<(), Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO user_costumes (account_id, costume_id, acquired_time, read_time) VALUES (?, ?, ?, 0)",
        )
        .bind(uid)
        .bind(costume_id)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    pub async fn grant_sd_costume(
        &self,
        uid: &str,
        sd_costume_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO user_sd_costumes (account_id, sd_costume_id, acquired_time, read_time) VALUES (?, ?, ?, 0)",
        )
        .bind(uid)
        .bind(sd_costume_id)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// gacha point balance (stored as an item row).
    pub async fn gacha_point_quantity(&self, uid: &str, point_id: &str) -> Result<i64, Error> {
        self.item_quantity(uid, point_id).await
    }

    pub async fn add_gacha_point(
        &self,
        uid: &str,
        point_id: &str,
        quantity: i64,
    ) -> Result<(), Error> {
        self.add_item(uid, point_id, quantity, unix_now()).await
    }

    /// record (or create) the draw counter for a gacha button.
    pub async fn bump_gacha_draw_count(
        &self,
        uid: &str,
        gacha_button_id: &str,
        count: i32,
    ) -> Result<(), Error> {
        let now = unix_now();
        sqlx::query(
            "INSERT INTO user_gacha_buttons (account_id, gacha_button_id, draw_count, last_draw_time) VALUES (?, ?, ?, ?)
             ON CONFLICT(account_id, gacha_button_id) DO UPDATE SET draw_count = draw_count + excluded.draw_count, last_draw_time = excluded.last_draw_time",
        )
        .bind(uid)
        .bind(gacha_button_id)
        .bind(count)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// record (or touch) a booth read; last_read_time is the latest read.
    pub async fn exchange_booth_read(
        &self,
        uid: &str,
        booth_id: &str,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_exchange_booths (account_id, exchange_booth_id, last_read_time) VALUES (?, ?, ?)
             ON CONFLICT(account_id, exchange_booth_id) DO UPDATE SET last_read_time = excluded.last_read_time",
        )
        .bind(uid)
        .bind(booth_id)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }

    /// how many times a user purchased one booth item (purchase-limit check).
    pub async fn exchange_purchase_count(
        &self,
        uid: &str,
        booth_id: &str,
        item_id: &str,
    ) -> Result<i64, Error> {
        let qty: Option<i64> = sqlx::query_scalar(
            "SELECT purchased_count FROM user_exchange_booth_purchases WHERE account_id = ? AND exchange_booth_id = ? AND booth_item_id = ?",
        )
        .bind(uid)
        .bind(booth_id)
        .bind(item_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(qty.unwrap_or(0))
    }

    /// record a booth purchase; purchased_count accumulates.
    pub async fn bump_exchange_purchase(
        &self,
        uid: &str,
        booth_id: &str,
        item_id: &str,
        qty: i64,
        now: i64,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_exchange_booth_purchases (account_id, exchange_booth_id, booth_item_id, purchased_count, last_purchased_time) VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(account_id, exchange_booth_id, booth_item_id) DO UPDATE SET purchased_count = purchased_count + excluded.purchased_count, last_purchased_time = excluded.last_purchased_time",
        )
        .bind(uid)
        .bind(booth_id)
        .bind(item_id)
        .bind(qty)
        .bind(now)
        .execute(self.pool())
        .await?;
        self.invalidate(uid);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;
    #[tokio::test]
    async fn consume_stones_free_first() {
        let dir = std::env::temp_dir().join(format!("symstones-{}", std::process::id()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let db = Database::open(&dir.join("test.db")).await.expect("open db");
        let uid = db
            .get_or_create_account("stones-test")
            .await
            .expect("account");
        async fn seed(db: &Database, uid: &str, free: i32, paid: i32) {
            sqlx::query("UPDATE user_balances SET free_quantity = ?, paid_quantity = ? WHERE account_id = ?")
                .bind(free).bind(paid).bind(uid)
                .execute(db.pool()).await.expect("seed");
        }

        // free (red) stones are spent first: an even split leaves both at 50
        seed(&db, &uid, 100, 100).await;
        assert!(db.consume_stones(&uid, 50, 50).await.unwrap());
        assert_eq!(db.balances(&uid).await.unwrap(), (50, 50));

        // a free-only request beyond the free balance spills into paid
        assert!(db.consume_stones(&uid, 100, 0).await.unwrap());
        assert_eq!(db.balances(&uid).await.unwrap(), (0, 0));

        // a mixed request with a short free balance spills the deficit to paid
        seed(&db, &uid, 50, 50).await;
        assert!(db.consume_stones(&uid, 60, 10).await.unwrap());
        assert_eq!(db.balances(&uid).await.unwrap(), (0, 30));

        // total-availability check still rejects over-spend
        seed(&db, &uid, 100, 100).await;
        assert!(!db.consume_stones(&uid, 250, 0).await.unwrap());
        assert_eq!(db.balances(&uid).await.unwrap(), (100, 100));
    }
}
