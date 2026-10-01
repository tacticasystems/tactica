import type { Access, Role } from "./types";

export const permissionDefinitions = [
  {
    bit: 1,
    label: "Administrator",
    description: "Grants every permission. Role hierarchy still applies.",
  },
  {
    bit: 2,
    label: "Manage unit",
    description: "Manage the unit profile and settings.",
    future: true,
  },
  { bit: 4, label: "Manage roles", description: "Create, edit, delete, and reorder roles." },
  { bit: 8, label: "Assign roles", description: "Assign and remove roles on members." },
  {
    bit: 16,
    label: "Manage ranks",
    description: "Create, edit, delete, and reorder ranks.",
  },
  { bit: 32, label: "Assign ranks", description: "Change member ranks." },
  {
    bit: 64,
    label: "Manage member profiles",
    description: "Edit member profile display fields.",
  },
];

export function canManageRoles(access: Access) {
  return access.is_owner || (access.permissions & 5) !== 0;
}

export function canAssignRole(access: Access, role: Role) {
  return (
    role.kind !== "everyone" &&
    (access.is_owner || (access.permissions & 9) !== 0) &&
    (access.is_owner || role.position < access.highest_role_position)
  );
}

export function canEditRole(access: Access, role: Role) {
  return (
    role.kind !== "administrator" &&
    canManageRoles(access) &&
    (access.is_owner || role.position < access.highest_role_position)
  );
}

export function canCreateRole(access: Access) {
  return canManageRoles(access) && (access.is_owner || access.highest_role_position > 0);
}

export function canDeleteRole(access: Access, role: Role) {
  return role.kind === "custom" && canEditRole(access, role);
}

export function canGrant(access: Access, bit: number) {
  return access.is_owner || (access.permissions & 1) !== 0 || (access.permissions & bit) !== 0;
}

export function canManageRanks(access: Access) {
  return access.is_owner || (access.permissions & 17) !== 0;
}

export function canAssignRanks(access: Access) {
  return access.is_owner || (access.permissions & 33) !== 0;
}

export function canManageMemberProfiles(access: Access) {
  return access.is_owner || (access.permissions & 65) !== 0;
}
