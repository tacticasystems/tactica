DROP TABLE unit_member_roles;
ALTER TABLE unit_roles DROP CONSTRAINT unit_roles_id_unit_unique;
ALTER TABLE unit_memberships DROP CONSTRAINT unit_memberships_id_unit_unique;
