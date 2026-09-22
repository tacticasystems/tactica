ALTER TABLE unit_roles
    DROP CONSTRAINT unit_roles_known_permissions,
    DROP CONSTRAINT unit_roles_position_unique,
    DROP COLUMN position;
