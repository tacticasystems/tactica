import { createContext, useContext } from "react";

import type { Unit, UnitDataSource } from "../lib/types";

interface WorkspaceContext {
  unit: Unit;
  source: UnitDataSource;
  preview: boolean;
  queryKey: readonly unknown[];
}
export const WorkspaceContext = createContext<WorkspaceContext | null>(null);

export function useWorkspace() {
  const value = useContext(WorkspaceContext);
  if (!value) throw new Error("A unit workspace is required.");
  return value;
}
