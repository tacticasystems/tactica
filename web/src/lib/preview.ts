import type { Member, Role, Unit, UnitDataSource } from "./types";

export const previewUnit: Unit = {
  id: "preview",
  slug: "9-rifles",
  display_name: "9 Rifles",
  icon_url: null,
  banner_url: null,
  biography: null,
  member_count: 6,
};

const roster = [
  ["Bull", "Lieutenant", "Lt"],
  ["Prongs", "Corporal", "Cpl"],
  ["J. Russell", "Corporal", "Cpl"],
  ["Young", "Lance Corporal", "LCpl"],
  ["Yegor", "Rifleman", "Rfn"],
  ["Dingo", "Recruit", "Rct"],
];

const members: Member[] = roster.map(([name, , abbreviation], index) => ({
  id: `member-${index}`,
  user_id: `user-${index}`,
  unit_id: "preview",
  rank_id: abbreviation,
  username: name,
  display_name: name,
  icon_url: null,
  role_ids: ["everyone"],
}));

let roles: Role[] = [
  {
    id: "admin",
    unit_id: "preview",
    display_name: "Administrator",
    description: null,
    permissions: 1,
    position: 2,
    kind: "administrator",
  },
  {
    id: "example",
    unit_id: "preview",
    display_name: "Example role",
    description: "An unassigned role for trying the editor.",
    permissions: 0,
    position: 1,
    kind: "custom",
  },
  {
    id: "everyone",
    unit_id: "preview",
    display_name: "Everyone",
    description: null,
    permissions: 0,
    position: 0,
    kind: "everyone",
  },
];

const copy = <T>(value: T): Promise<T> => Promise.resolve(structuredClone(value));
const bindings = new Map<string, Set<string>>();

export const previewApi: UnitDataSource = {
  allMembers: () => copy(membersWithRoles()),
  roleMembers: (_unitId, roleId) =>
    copy(
      roleId === "everyone"
        ? members.map((member) => member.id)
        : [...(bindings.get(roleId) ?? [])],
    ),
  async setRoleMember(_unitId, roleId, memberId, assigned) {
    if (
      !roles.some((role) => role.id === roleId) ||
      !members.some((member) => member.id === memberId)
    )
      throw new Error("This role or member no longer exists.");
    if (roleId === "everyone") throw new Error("Everyone applies automatically.");
    const memberIds = bindings.get(roleId) ?? new Set<string>();
    if (assigned) memberIds.add(memberId);
    else memberIds.delete(memberId);
    bindings.set(roleId, memberIds);
  },
  async reorderRoles(_unitId, roleIds) {
    if (
      roleIds.length !== roles.length ||
      new Set(roleIds).size !== roles.length ||
      roleIds.some((id) => !roles.some((role) => role.id === id))
    )
      throw new Error("The role list changed. Reload it and try again.");
    if (
      roles.find((role) => role.kind === "administrator")?.id !== roleIds[0] ||
      roles.find((role) => role.kind === "everyone")?.id !== roleIds[roleIds.length - 1]
    )
      throw new Error("Built-in roles cannot be moved.");
    const reordered = roleIds.map((id, index) => {
      const role = roles.find((item) => item.id === id);
      if (!role) throw new Error("This role no longer exists.");
      return { ...role, position: roleIds.length - index - 1 };
    });
    roles = reordered;
    return structuredClone(roles);
  },
  async deleteRole(_unitId, roleId) {
    const role = roles.find((item) => item.id === roleId);
    if (!role) throw new Error("This role no longer exists.");
    if (role.kind !== "custom") throw new Error("Built-in roles cannot be deleted.");
    bindings.delete(roleId);
    roles = roles
      .filter((item) => item.id !== roleId)
      .map((item) =>
        item.position > role.position ? { ...item, position: item.position - 1 } : item,
      );
  },
  unit: () => copy(previewUnit),
  members: (_id, offset) => copy(membersWithRoles().slice(offset, offset + 20)),
  ranks: () =>
    copy(
      Array.from(
        new Map(
          roster.map(([, name, abbreviation]) => [
            abbreviation,
            {
              id: abbreviation,
              unit_id: "preview",
              slug: abbreviation,
              display_name: name,
              icon_url: null,
              description: null,
            },
          ]),
        ).values(),
      ),
    ),
  roles: () => copy(roles),
  // Capabilities simulate an editor; they make no claim about the example's owner.
  access: () =>
    copy({
      member_id: "preview-editor",
      is_owner: true,
      permissions: 127,
      highest_role_position: 2,
    }),
  async saveRole(_unitId, roleId, input) {
    const existing = roles.find((role) => role.id === roleId);
    if (roles.some((role) => role.id !== roleId && role.display_name === input.display_name)) {
      throw new Error("A role with this name already exists. Choose another name.");
    }
    if (existing) {
      const saved = { ...existing, ...input };
      roles = roles.map((role) => (role.id === roleId ? saved : role));
      return structuredClone(saved);
    }
    const next: Role = {
      id: crypto.randomUUID(),
      unit_id: "preview",
      display_name: input.display_name ?? "",
      description: input.description ?? null,
      permissions: input.permissions,
      kind: "custom",
      position: 1,
    };
    roles = roles.map((role) =>
      role.kind !== "everyone" ? { ...role, position: role.position + 1 } : role,
    );
    roles.push(next);
    roles.sort((a, b) => b.position - a.position);
    return structuredClone(next);
  },
};

function membersWithRoles(): Member[] {
  return members.map((member) => ({
    ...member,
    role_ids: roles
      .filter((role) => role.kind === "everyone" || bindings.get(role.id)?.has(member.id))
      .map((role) => role.id),
  }));
}
