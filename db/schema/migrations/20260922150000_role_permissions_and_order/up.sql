-- This application is not deployed; existing opaque permission data is not migrated.
ALTER TABLE unit_roles
    ADD COLUMN position BIGINT NOT NULL CHECK (position >= 0),
    ADD CONSTRAINT unit_roles_known_permissions CHECK (permissions >= 0 AND permissions <= 127),
    ADD CONSTRAINT unit_roles_position_unique UNIQUE (unit_id, position) DEFERRABLE INITIALLY DEFERRED;
