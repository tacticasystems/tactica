ALTER TABLE unit_ranks ADD COLUMN position BIGINT;

-- Preserve the previous ID order for existing units (highest first).
WITH ordered AS (
    SELECT id, row_number() OVER (PARTITION BY unit_id ORDER BY id DESC) - 1 AS position
    FROM unit_ranks
)
UPDATE unit_ranks SET position = ordered.position FROM ordered WHERE unit_ranks.id = ordered.id;

ALTER TABLE unit_ranks
    ALTER COLUMN position SET NOT NULL,
    ADD CONSTRAINT unit_ranks_position_nonnegative CHECK (position >= 0),
    ADD CONSTRAINT unit_ranks_position_unique UNIQUE (unit_id, position) DEFERRABLE INITIALLY DEFERRED;
