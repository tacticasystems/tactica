export interface User {
  id: string;
  username: string;
  display_name: string | null;
  icon_url: string | null;
}

export interface Unit {
  id: string;
  slug: string;
  display_name: string;
  icon_url: string | null;
  banner_url: string | null;
  biography: string | null;
  member_count: number;
}

export interface CreatedUnit extends Omit<Unit, "member_count"> {
  rank_id: string;
  member_id: string;
}

export interface Member {
  id: string;
  user_id: string;
  unit_id: string;
  rank_id: string;
  username: string;
  display_name: string | null;
  icon_url: string | null;
  role_ids: string[];
}

export interface Rank {
  id: string;
  unit_id: string;
  slug: string;
  display_name: string | null;
  icon_url: string | null;
  description: string | null;
}

export interface Role {
  id: string;
  unit_id: string;
  display_name: string;
  description: string | null;
  permissions: number;
  position: number;
  kind: "custom" | "administrator" | "everyone";
}

export interface Access {
  member_id: string;
  is_owner: boolean;
  permissions: number;
  highest_role_position: number;
}

export interface RoleInput {
  display_name?: string;
  description?: string | null;
  permissions: number;
}

export interface UnitDataSource {
  unit(id: string, signal?: AbortSignal): Promise<Unit>;
  members(id: string, offset: number, signal?: AbortSignal): Promise<Member[]>;
  ranks(id: string, signal?: AbortSignal): Promise<Rank[]>;
  roles(id: string, signal?: AbortSignal): Promise<Role[]>;
  access(id: string, signal?: AbortSignal): Promise<Access>;
  allMembers(id: string, signal?: AbortSignal): Promise<Member[]>;
  roleMembers(unitId: string, roleId: string, signal?: AbortSignal): Promise<string[]>;
  setRoleMember(unitId: string, roleId: string, memberId: string, assigned: boolean): Promise<void>;
  saveRole(unitId: string, roleId: string | null, input: RoleInput): Promise<Role>;
  deleteRole(unitId: string, roleId: string): Promise<void>;
  /** Role IDs in display order, highest first. */
  reorderRoles(unitId: string, roleIds: string[]): Promise<Role[]>;
}
