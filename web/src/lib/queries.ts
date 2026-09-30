import { QueryClient, queryOptions } from "@tanstack/react-query";
import { ApiError } from "./session-client";
import { currentUser, myUnits, sessionSnapshot } from "./api";
import type { UnitDataSource } from "./types";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      // Session rotation and authorization failures must never be retried.
      retry: false,
    },
    mutations: { retry: false },
  },
});

export const userOptions = () =>
  queryOptions({
    queryKey: ["session", sessionSnapshot(), "me"],
    queryFn: ({ signal }) => currentUser(signal),
  });
export const unitsOptions = () =>
  queryOptions({
    queryKey: ["session", sessionSnapshot(), "units"],
    queryFn: ({ signal }) => myUnits(signal),
  });
export function unitOptions(source: UnitDataSource, unitId: string) {
  return queryOptions({
    queryKey: ["session", unitId === "preview" ? "preview" : sessionSnapshot(), "unit", unitId],
    queryFn: ({ signal }) => source.unit(unitId, signal),
  });
}

export function errorMessage(error: unknown) {
  if (error instanceof ApiError && error.status === 403)
    return "Your permissions changed, or this role is above your role. Reload the roles and try again.";
  return error instanceof Error ? error.message : "The change could not be saved. Try again.";
}
