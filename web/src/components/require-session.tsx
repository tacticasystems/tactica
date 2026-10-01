import { Navigate } from "@tanstack/react-router";
import { useSyncExternalStore, type ReactNode } from "react";

import { sessionSnapshot, subscribeSession } from "../lib/api";

export function RequireSession({ children }: { children: ReactNode }) {
  const session = useSyncExternalStore(subscribeSession, sessionSnapshot);
  return session ? children : <Navigate to="/login" replace />;
}
