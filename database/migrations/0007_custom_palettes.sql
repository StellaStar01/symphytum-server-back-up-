-- saved custom palettes (3NF child of accounts); image served by hattp
CREATE TABLE user_custom_palettes (
    account_id     TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    number         INTEGER NOT NULL,
    image_url      TEXT NOT NULL,
    is_inactivated INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, number)
);
