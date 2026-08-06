-- jump rope progress (3NF children of accounts)
CREATE TABLE user_jump_ropes (
    account_id             TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    jump_rope_id           TEXT NOT NULL,
    best_jump_count        INTEGER NOT NULL DEFAULT 0,
    is_cleared             INTEGER NOT NULL DEFAULT 0,
    play_count             INTEGER NOT NULL DEFAULT 0,
    notification_read_time INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, jump_rope_id)
);

CREATE TABLE user_jump_rope_npc_exits (
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    jump_rope_id TEXT NOT NULL,
    exit_index   INTEGER NOT NULL,
    jump_count   INTEGER NOT NULL,
    PRIMARY KEY (account_id, jump_rope_id, exit_index)
);
