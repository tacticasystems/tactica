ALTER TABLE files ADD COLUMN is_banner BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE files ADD CONSTRAINT files_single_artwork_kind CHECK (NOT (is_icon AND is_banner));
