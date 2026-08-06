-- received player level rewards (3NF child of user_parks)
CREATE TABLE user_park_level_rewards (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    level      INTEGER NOT NULL,
    PRIMARY KEY (account_id, level)
);

-- every account is created maxed; mark all level rewards as received so the level-up popup never shows.
INSERT INTO user_park_level_rewards (account_id, level)
SELECT a.id, lv.x
FROM accounts a
CROSS JOIN (
    WITH RECURSIVE seq(x) AS (
        SELECT 1 UNION ALL SELECT x + 1 FROM seq WHERE x < 50
    )
    SELECT * FROM seq
) lv;
