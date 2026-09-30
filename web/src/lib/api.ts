import { SessionClient } from "./session-client";
import type { Session, SessionStore } from "./session-client";
import type { Access, CreatedUnit, Member, Rank, Role, Unit, UnitDataSource, User } from "./types";

const sessionKey = "tactica.session.v1";
const event = "tactica:session";
// Browsers without Web Locks keep independent tab sessions to avoid replay.
const storage = navigator.locks ? localStorage : sessionStorage;

export const sessionStore: SessionStore = {
  read() {
    try {
      const value: unknown = JSON.parse(storage.getItem(sessionKey) ?? "null");
      if (
        typeof value === "object" &&
        value !== null &&
        "access_token" in value &&
        typeof value.access_token === "string" &&
        "refresh_token" in value &&
        typeof value.refresh_token === "string" &&
        "expires_at" in value &&
        typeof value.expires_at === "number" &&
        "session_id" in value &&
        typeof value.session_id === "string"
      ) {
        return value as Session;
      }
    } catch {
      /* A corrupt or unavailable storage entry is a signed-out session. */
    }
    return null;
  },
  write(session) {
    if (session) storage.setItem(sessionKey, JSON.stringify(session));
    else storage.removeItem(sessionKey);
    window.dispatchEvent(new Event(event));
  },
};

export function subscribeSession(callback: () => void) {
  const onStorage = (change: StorageEvent) => {
    if (change.key === sessionKey || change.key === null) callback();
  };
  window.addEventListener(event, callback);
  window.addEventListener("storage", onStorage);
  return () => {
    window.removeEventListener(event, callback);
    window.removeEventListener("storage", onStorage);
  };
}

export const sessionSnapshot = () => sessionStore.read()?.session_id ?? null;
export const client = new SessionClient(
  sessionStore,
  fetch,
  async (work) =>
    await (navigator.locks ? navigator.locks.request("tactica.session", work) : work()),
);

async function listAll<T>(path: string, key: string, signal?: AbortSignal): Promise<T[]> {
  const items: T[] = [];
  for (let offset = 0; ; offset += 100) {
    const response = await client.request<Record<string, T[]>>(
      `${path}?offset=${offset}&limit=100`,
      { signal },
    );
    const page = response[key];
    if (!page) throw new Error("Tactica returned an invalid collection.");
    items.push(...page);
    if (page.length < 100) return items;
  }
}

export const currentUser = (signal?: AbortSignal) => client.request<User>("/auth/me", { signal });
export const myUnits = (signal?: AbortSignal) => listAll<Unit>("/auth/me/units", "units", signal);
export const createUnit = (display_name: string, slug: string) =>
  client.request<CreatedUnit>("/units", {
    method: "POST",
    body: JSON.stringify({ display_name, slug, icon_url: null, banner_url: null, biography: null }),
  });

export const api: UnitDataSource = {
  allMembers: (id, signal) => listAll<Member>(`/units/${id}/members`, "members", signal),
  roleMembers: (unitId, roleId, signal) =>
    listAll<string>(`/units/${unitId}/roles/${roleId}/members`, "member_ids", signal),
  setRoleMember: (unitId, roleId, memberId, assigned) =>
    client.request<void>(`/units/${unitId}/members/${memberId}/roles/${roleId}`, {
      method: assigned ? "PUT" : "DELETE",
    }),
  reorderRoles: async (unitId, roleIds) =>
    (
      await client.request<{ roles: Role[] }>(`/units/${unitId}/roles/order`, {
        method: "PATCH",
        // The API takes lowest first; the UI displays highest first.
        body: JSON.stringify({ role_ids: [...roleIds].reverse() }),
      })
    ).roles,
  deleteRole: (unitId, roleId) =>
    client.request<void>(`/units/${unitId}/roles/${roleId}`, { method: "DELETE" }),
  unit: (id, signal) => client.request<Unit>(`/units/${id}`, { signal }),
  members: async (id, offset, signal) =>
    (
      await client.request<{ members: Member[] }>(
        `/units/${id}/members?offset=${offset}&limit=20`,
        { signal },
      )
    ).members,
  ranks: (id, signal) => listAll<Rank>(`/units/${id}/ranks`, "ranks", signal),
  roles: (id, signal) => listAll<Role>(`/units/${id}/roles`, "roles", signal),
  access: (id, signal) => client.request<Access>(`/units/${id}/access`, { signal }),
  saveRole: (unitId, roleId, input) =>
    client.request<Role>(`/units/${unitId}/roles${roleId ? `/${roleId}` : ""}`, {
      method: roleId ? "PATCH" : "POST",
      body: JSON.stringify(input),
    }),
};
