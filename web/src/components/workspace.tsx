import { useQuery } from "@tanstack/react-query";
import { Navigate, Outlet, useLocation, useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { api } from "../lib/api";
import { previewApi } from "../lib/preview";
import { unitOptions, userOptions } from "../lib/queries";

import { AppShell } from "./app-shell";
import { ErrorState } from "./error-state";
import { LoadingState } from "./loading-state";
import { RequireSession } from "./require-session";
import { WorkspaceContext } from "./workspace-context";

export function UnitWorkspace() {
  const { unitId } = useParams({ strict: false });
  if (!unitId) return <Navigate to="/units" replace />;
  return unitId === "preview" ? (
    <LoadedWorkspace unitId={unitId} />
  ) : (
    <RequireSession>
      <LoadedWorkspace unitId={unitId} />
    </RequireSession>
  );
}

function LoadedWorkspace({ unitId }: { unitId: string }) {
  const preview = unitId === "preview";
  const source = preview ? previewApi : api;
  const options = unitOptions(source, unitId);
  const query = useQuery(options);
  const user = useQuery({ ...userOptions(), enabled: !preview });
  const location = useLocation();
  const label = location.pathname.endsWith("/ranks")
    ? "Ranks"
    : location.pathname.endsWith("/roles")
      ? "Roles"
      : location.pathname.endsWith("/profile")
        ? "Profile"
        : location.pathname.endsWith("/overview")
          ? "Overview"
          : "Personnel";

  useEffect(() => {
    document.title = `${label}${query.data ? ` · ${query.data.display_name}` : ""} · Tactica`;
  }, [label, query.data]);

  return (
    <AppShell
      unit={query.data}
      preview={preview}
      accountName={preview ? "Preview" : (user.data?.display_name ?? user.data?.username)}
      label={label}
    >
      {query.isPending ? (
        <LoadingState />
      ) : query.isError ? (
        <ErrorState error={query.error} retry={() => void query.refetch()} />
      ) : (
        <WorkspaceContext.Provider
          value={{ unit: query.data, source, preview, queryKey: options.queryKey }}
        >
          <Outlet key={unitId} />
        </WorkspaceContext.Provider>
      )}
    </AppShell>
  );
}
