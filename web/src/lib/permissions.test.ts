import { describe, expect, it } from "vitest";
import { canAssignRole, canCreateRole, canDeleteRole, canEditRole, canGrant } from "./permissions";
import type { Access, Role } from "./types";

const access: Access = {
  member_id: "member",
  is_owner: false,
  permissions: 4,
  highest_role_position: 2,
};
const role: Role = {
  id: "role",
  unit_id: "unit",
  display_name: "Custom",
  description: null,
  permissions: 0,
  position: 1,
  kind: "custom",
};

describe("UI role capabilities", () => {
  it("keeps role assignment independent of role editing and enforces hierarchy", () => {
    expect(canAssignRole(access, role)).toBe(false);
    const assigner = { ...access, permissions: 8 };
    expect(canAssignRole(assigner, role)).toBe(true);
    expect(canEditRole(assigner, role)).toBe(false);
    expect(canAssignRole(assigner, { ...role, position: 2 })).toBe(false);
    expect(canAssignRole(assigner, { ...role, position: 3 })).toBe(false);
    expect(canAssignRole({ ...assigner, permissions: 1 }, role)).toBe(true);
    expect(canAssignRole({ ...assigner, permissions: 1 }, { ...role, position: 2 })).toBe(false);
  });
  it("lets only the owner assign the top Administrator and never assigns Everyone", () => {
    const adminRole = { ...role, kind: "administrator" as const, position: 3 };
    expect(canAssignRole({ ...access, permissions: 127 }, adminRole)).toBe(false);
    const owner = { ...access, is_owner: true };
    expect(canAssignRole(owner, adminRole)).toBe(true);
    expect(canAssignRole(owner, { ...role, kind: "everyone" })).toBe(false);
    expect(canAssignRole({ ...access, permissions: 127 }, { ...role, kind: "everyone" })).toBe(
      false,
    );
  });
  it("allows deletion only for manageable custom roles", () => {
    expect(canDeleteRole(access, role)).toBe(true);
    expect(canDeleteRole(access, { ...role, position: 2 })).toBe(false);
    expect(canDeleteRole({ ...access, permissions: 0 }, role)).toBe(false);
    const owner = { ...access, is_owner: true };
    expect(canDeleteRole(owner, { ...role, kind: "administrator" })).toBe(false);
    expect(canDeleteRole(owner, { ...role, kind: "everyone" })).toBe(false);
  });
  it("keeps administrator permissions subject to the role hierarchy", () => {
    const admin = { ...access, permissions: 127 };
    expect(canEditRole(admin, role)).toBe(true);
    expect(canEditRole(admin, { ...role, position: 2 })).toBe(false);
    expect(canEditRole(admin, { ...role, position: 3 })).toBe(false);
  });
  it("protects Administrator even for an owner, while allowing Everyone permission edits", () => {
    const owner = { ...access, is_owner: true };
    expect(canEditRole(owner, { ...role, kind: "administrator" })).toBe(false);
    expect(canEditRole(owner, { ...role, kind: "everyone", position: 0 })).toBe(true);
  });
  it("does not offer role creation above a caller with only Everyone", () => {
    expect(canCreateRole({ ...access, highest_role_position: 0 })).toBe(false);
    expect(canCreateRole({ ...access, highest_role_position: 0, is_owner: true })).toBe(true);
  });
  it("does not let ordinary permission holders grant Administrator", () => {
    expect(canGrant({ ...access, permissions: 126 }, 1)).toBe(false);
    expect(canGrant({ ...access, permissions: 126 }, 64)).toBe(true);
  });
});
