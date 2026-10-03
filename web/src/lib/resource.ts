import type { FunctionComponent, ReactNode } from "react";

export interface ResourceField<T, Context = undefined> {
  id: string;
  label: string;
  in: readonly ("list" | "view" | "edit")[];
  read: (record: T, context: Context) => ReactNode;
  /** Include "edit" in `in` to opt into the automatic text editor. */
  edit?: { value: (record: T) => string; write: (value: string) => Partial<T>; required?: boolean };
}

export interface ResourceSource<T> {
  list: (offset: number, signal?: AbortSignal) => Promise<T[]>;
  get: (id: string, signal?: AbortSignal) => Promise<T>;
  update?: (id: string, input: Partial<T>) => Promise<unknown>;
  remove?: (id: string) => Promise<unknown>;
}

export interface ResourceState<T, Context> {
  source: ResourceSource<T>;
  queryKey: readonly unknown[];
  context: Context;
  description?: string;
  caption?: string;
  total?: number;
  /** Related data required by field renderers. */
  pending?: boolean;
  error?: unknown;
  refetch?: () => void;
  canEdit?: (record: T) => boolean;
  canDelete?: (record: T) => boolean;
}

export interface ResourceDefinition<T extends { id: string }, Context = undefined> {
  apiBase: string;
  /** Collection cache key; defaults to the last segment of apiBase. */
  key?: string;
  itemKey?: string;
  label: string;
  pluralLabel: string;
  name: (record: T) => string;
  fields: readonly ResourceField<T, Context>[];
  searchText?: (record: T) => string;
  /** Turn off when a field already renders its own detail link. */
  listActions?: boolean;
  /** Session/preview scope, dependencies and permissions stay with the resource. */
  useResource: () => ResourceState<T, Context>;
  listView?: FunctionComponent;
  viewView?: FunctionComponent;
  editView?: FunctionComponent;
  deleteView?: FunctionComponent;
}

export function resolveResourcePath(template: string, params: Record<string, string>) {
  return template.replace(/\{([^}]+)\}/g, (_match, key: string) => {
    if (params[key] === undefined) throw new Error(`Missing resource parameter: ${key}`);
    return encodeURIComponent(params[key]);
  });
}

/** Bind request to the existing session client, or supply a fake transport. */
export function createResourceSource<T>(
  apiBase: string,
  params: Record<string, string>,
  collectionKey: string,
  request: <R>(path: string, init?: RequestInit) => Promise<R>,
): ResourceSource<T> {
  const base = resolveResourcePath(apiBase, params);
  const item = (id: string) => `${base}/${encodeURIComponent(id)}`;
  return {
    list: async (offset, signal) => {
      const result = await request<Record<string, T[]>>(`${base}?offset=${offset}&limit=20`, {
        signal,
      });
      if (!Array.isArray(result[collectionKey]))
        throw new Error("Tactica returned an invalid collection.");
      return result[collectionKey];
    },
    get: (id, signal) => request<T>(item(id), { signal }),
    update: (id, input) => request(item(id), { method: "PATCH", body: JSON.stringify(input) }),
    remove: (id) => request(item(id), { method: "DELETE" }),
  };
}
