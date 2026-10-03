import { useQuery } from "@tanstack/react-query";
import { Link, Navigate, Outlet, useLocation, useParams } from "@tanstack/react-router";
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
  const { rankId } = useParams({ strict: false });
  const ranks = useQuery({
    queryKey: [...options.queryKey, "ranks"],
    queryFn: ({ signal }) => source.ranks(unitId, signal),
    enabled: !!rankId && query.isSuccess,
  });
  const rank = rankId ? ranks.data?.find((item) => item.id === rankId) : undefined;
  const label = rankId
    ? (rank?.display_name ?? rank?.slug ?? "Rank")
    : /\/members\/[^/]+\/edit$/.test(location.pathname)
      ? "Edit member"
      : /\/members\/[^/]+$/.test(location.pathname)
        ? "Member"
        : location.pathname.endsWith("/ranks") || /\/ranks\/[^/]+$/.test(location.pathname)
          ? "Ranks"
          : location.pathname.endsWith("/roles")
            ? "Roles"
            : location.pathname.endsWith("/profile")
              ? "Profile"
              : location.pathname.endsWith("/overview")
                ? "Overview"
                : "Members";

  useEffect(() => {
    document.title = `${label}${query.data ? ` · ${query.data.display_name}` : ""} · Tactica`;
  }, [label, query.data]);

  return (
    <AppShell
      unit={query.data}
      preview={preview}
      accountName={preview ? "Preview" : (user.data?.display_name ?? user.data?.username)}
      label={label}
      parentBreadcrumb={
        rankId && (
          <Link to="/units/$unitId/ranks" params={{ unitId }} activeOptions={{ exact: true }}>
            Ranks
          </Link>
        )
      }
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
