ALTER TABLE units ADD COLUMN owner_id UUID NOT NULL REFERENCES users (id) ON DELETE RESTRICT;
CREATE INDEX units_owner_id_idx ON units (owner_id);
