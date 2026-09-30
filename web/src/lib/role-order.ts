import { canEditRole } from "./permissions";
import type { Access, Role } from "./types";

/** Moves within the manageable custom-role range; IDs remain highest first. */
export function moveRole(
  access: Access,
  roles: Role[],
  roleId: string,
  targetId: string,
): string[] | null {
  const from = roles.findIndex((role) => role.id === roleId);
  const to = roles.findIndex((role) => role.id === targetId);
  if (from < 0 || to < 0 || from === to) return null;
  const crossed = roles.slice(Math.min(from, to), Math.max(from, to) + 1);
  if (crossed.some((role) => role.kind !== "custom" || !canEditRole(access, role))) return null;
  const order = roles.map((role) => role.id);
  order.splice(from, 1);
  order.splice(to, 0, roleId);
  return order;
}
