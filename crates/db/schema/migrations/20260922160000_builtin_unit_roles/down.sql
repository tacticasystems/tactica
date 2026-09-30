DROP INDEX unit_roles_builtin_unique;
ALTER TABLE unit_roles
    DROP CONSTRAINT unit_roles_administrator_fixed,
    DROP CONSTRAINT unit_roles_everyone_bottom,
    DROP CONSTRAINT unit_roles_everyone_fixed,
    DROP COLUMN kind;
