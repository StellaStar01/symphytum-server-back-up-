-- palette parts (3NF child of user_custom_palettes)
CREATE TABLE user_custom_palette_parts (
    account_id         TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    number             INTEGER NOT NULL,
    part_index         INTEGER NOT NULL,
    resource_type      INTEGER NOT NULL,
    resource_id        TEXT NOT NULL,
    position_x_permil  INTEGER NOT NULL,
    position_y_permil  INTEGER NOT NULL,
    scale_permil       INTEGER NOT NULL,
    rotation_permil    INTEGER NOT NULL,
    layer              INTEGER NOT NULL,
    PRIMARY KEY (account_id, number, part_index)
);
