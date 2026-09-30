import { describe, expect, it } from "vitest";
import { moveRole } from "./role-order";
import type { Access, Role } from "./types";

const access: Access = {
  member_id: "member",
  is_owner: false,
  permissions: 4,
  highest_role_position: 3,
};
const roles: Role[] = [
  {
    id: "admin",
    unit_id: "unit",
    display_name: "Administrator",
    description: null,
    permissions: 1,
    position: 4,
    kind: "administrator",
  },
  {
    id: "senior",
    unit_id: "unit",
    display_name: "Senior",
    description: null,
    permissions: 4,
    position: 3,
    kind: "custom",
  },
  {
    id: "a",
    unit_id: "unit",
    display_name: "A",
    description: null,
    permissions: 0,
    position: 2,
    kind: "custom",
  },
  {
    id: "b",
    unit_id: "unit",
    display_name: "B",
    description: null,
    permissions: 0,
    position: 1,
    kind: "custom",
  },
  {
    id: "everyone",
    unit_id: "unit",
    display_name: "Everyone",
    description: null,
    permissions: 0,
    position: 0,
    kind: "everyone",
  },
];

describe("role ordering", () => {
  it("swaps adjacent manageable roles without changing the source list", () => {
    expect(moveRole(access, roles, "b", "a")).toEqual(["admin", "senior", "b", "a", "everyone"]);
    expect(moveRole(access, roles, "a", "b")).toEqual(["admin", "senior", "b", "a", "everyone"]);
    expect(roles.map((role) => role.id)).toEqual(["admin", "senior", "a", "b", "everyone"]);
  });
  it("keeps both built-ins fixed even for the owner", () => {
    const owner = { ...access, is_owner: true };
    expect(moveRole(owner, roles, "senior", "admin")).toBeNull();
    expect(moveRole(owner, roles, "b", "everyone")).toBeNull();
    expect(moveRole(owner, roles, "admin", "senior")).toBeNull();
    expect(moveRole(owner, roles, "everyone", "b")).toBeNull();
  });
  it("prevents crossing or moving the caller's highest role", () => {
    expect(moveRole(access, roles, "a", "senior")).toBeNull();
    expect(moveRole(access, roles, "senior", "a")).toBeNull();
    expect(moveRole({ ...access, is_owner: true }, roles, "a", "senior")).not.toBeNull();
  });
  it("allows longer drags only when every crossed role is manageable", () => {
    expect(moveRole({ ...access, is_owner: true }, roles, "b", "senior")).toEqual([
      "admin",
      "b",
      "senior",
      "a",
      "everyone",
    ]);
    expect(moveRole(access, roles, "b", "senior")).toBeNull();
    expect(moveRole(access, roles, "b", "b")).toBeNull();
  });
  it("requires role management and rejects missing roles", () => {
    expect(moveRole({ ...access, permissions: 0 }, roles, "b", "a")).toBeNull();
    expect(moveRole(access, roles, "missing", "a")).toBeNull();
  });
});
