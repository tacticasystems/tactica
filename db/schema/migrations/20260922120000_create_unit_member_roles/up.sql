ALTER TABLE unit_memberships ADD CONSTRAINT unit_memberships_id_unit_unique UNIQUE (id, unit_id);
ALTER TABLE unit_roles ADD CONSTRAINT unit_roles_id_unit_unique UNIQUE (id, unit_id);

CREATE TABLE unit_member_roles (
    member_id UUID NOT NULL,
    role_id UUID NOT NULL,
    unit_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (member_id, role_id),
    FOREIGN KEY (member_id, unit_id) REFERENCES unit_memberships (id, unit_id) ON DELETE CASCADE,
    FOREIGN KEY (role_id, unit_id) REFERENCES unit_roles (id, unit_id) ON DELETE CASCADE
);

CREATE INDEX unit_member_roles_role_id_idx ON unit_member_roles (role_id);
CREATE INDEX unit_member_roles_unit_id_idx ON unit_member_roles (unit_id);
