import { Link } from "@tanstack/react-router";

import type { Member, Role } from "../lib/types";

export function MemberRolePills({
  member,
  roles,
  unitId,
  linkRoles,
}: {
  member: Member;
  roles: Role[];
  unitId: string;
  linkRoles: boolean;
}) {
  const name = member.display_name ?? member.username;

  return (
    <ul className="member-role-pills" aria-label={`${name} roles`}>
      {roles
        .filter((role) => role.kind !== "everyone" && member.role_ids.includes(role.id))
        .map((role) => (
          <li key={role.id}>
            {linkRoles ? (
              <Link
                className="role-pill"
                to="/units/$unitId/roles"
                params={{ unitId: unitId }}
                search={{ roleId: role.id }}
              >
                {role.display_name}
              </Link>
            ) : (
              <span className="role-pill">{role.display_name}</span>
            )}
          </li>
        ))}
    </ul>
  );
}
