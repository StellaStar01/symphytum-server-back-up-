-- exchange booth state (3NF children of accounts)
CREATE TABLE user_exchange_booths (
    account_id        TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    exchange_booth_id TEXT NOT NULL,
    last_read_time    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, exchange_booth_id)
);

CREATE TABLE user_exchange_booth_purchases (
    account_id          TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    exchange_booth_id   TEXT NOT NULL,
    booth_item_id       TEXT NOT NULL,
    purchased_count     INTEGER NOT NULL DEFAULT 0,
    last_purchased_time INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, exchange_booth_id, booth_item_id)
);
