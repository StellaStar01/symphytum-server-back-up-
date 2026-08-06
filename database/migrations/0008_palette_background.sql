-- palette background card for the custom-palette list response
ALTER TABLE user_custom_palettes ADD COLUMN background_card_id TEXT NOT NULL DEFAULT '';
ALTER TABLE user_custom_palettes ADD COLUMN background_card_potential_upgrade_count INTEGER NOT NULL DEFAULT 0;
