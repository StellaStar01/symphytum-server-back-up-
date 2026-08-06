-- notice read state (3NF child of accounts)
CREATE TABLE user_notice_read_times (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    notice_id  TEXT NOT NULL,
    read_time  INTEGER NOT NULL,
    PRIMARY KEY (account_id, notice_id)
);
