ALTER TABLE unit_roles
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'custom'
        CHECK (kind IN ('custom', 'administrator', 'everyone')),
    ADD CONSTRAINT unit_roles_administrator_fixed
        CHECK (kind <> 'administrator' OR
            (display_name = 'Administrator' AND description IS NULL AND permissions = 1)),
    ADD CONSTRAINT unit_roles_everyone_fixed
        CHECK (kind <> 'everyone' OR (display_name = 'Everyone' AND description IS NULL)),
    ADD CONSTRAINT unit_roles_everyone_bottom
        CHECK ((kind = 'everyone') = (position = 0));

CREATE UNIQUE INDEX unit_roles_builtin_unique ON unit_roles (unit_id, kind)
    WHERE kind <> 'custom';
