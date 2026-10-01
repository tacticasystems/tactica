import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { useState } from "react";

import { canManageRoles } from "../lib/permissions";

import { EmptyState } from "../components/empty-state";
import { ErrorState } from "../components/error-state";
import { LoadingState } from "../components/loading-state";
import { PageHeading } from "../components/page-heading";
import { PersonnelTable } from "../components/personnel-table";
import { RosterPagination } from "../components/roster-pagination";
import { Input } from "../components/ui/input";
import { useWorkspace } from "../components/workspace-context";

export function PersonnelPage() {
  const { unit, source, preview, queryKey } = useWorkspace();
  const [offset, setOffset] = useState(0);
  const [search, setSearch] = useState("");
  const members = useQuery({
    queryKey: [...queryKey, "members", offset],
    queryFn: ({ signal }) => source.members(unit.id, offset, signal),
    placeholderData: keepPreviousData,
  });

  const ranks = useQuery({
    queryKey: [...queryKey, "ranks"],
    queryFn: ({ signal }) => source.ranks(unit.id, signal),
  });

  const roles = useQuery({
    queryKey: [...queryKey, "roles"],
    queryFn: ({ signal }) => source.roles(unit.id, signal),
  });

  const access = useQuery({
    queryKey: [...queryKey, "access"],
    queryFn: ({ signal }) => source.access(unit.id, signal),
  });

  const linkRoles = !!access.data && canManageRoles(access.data);

  const filtered = members.data?.filter((member) =>
    (member.display_name ?? member.username)
      .toLocaleLowerCase()
      .includes(search.trim().toLocaleLowerCase()),
  );

  return (
    <>
      <PageHeading
        title="Members"
        description={`The ${unit.display_name} roster.`}
        action={
          <label className="search-input">
            <Search size={17} aria-hidden="true" />
            <span className="sr-only">Search this page</span>
            <Input
              type="search"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder={unit.member_count <= 20 ? "Search members" : "Search this page"}
            />
          </label>
        }
      />
      <div className="roster-meta">
        <span>
          {unit.member_count} {unit.member_count === 1 ? "member" : "members"}
        </span>
        {search && <span>{filtered?.length ?? 0} on this page match</span>}
        {members.isFetching && !members.isPending && <span role="status">Updating…</span>}
      </div>
      {members.isPending || ranks.isPending || roles.isPending ? (
        <LoadingState label="Loading personnel" />
      ) : members.isError || ranks.isError || roles.isError ? (
        <ErrorState
          error={members.error ?? ranks.error ?? roles.error}
          retry={() => {
            void members.refetch();
            void ranks.refetch();
            void roles.refetch();
          }}
        />
      ) : filtered?.length === 0 ? (
        <EmptyState title={search ? "No matching members on this page" : "No members on this page"}>
          {search
            ? "Try another name or clear your search."
            : "Choose the previous page if the roster has changed."}
        </EmptyState>
      ) : (
        <PersonnelTable
          members={filtered ?? []}
          ranks={ranks.data ?? []}
          roles={roles.data ?? []}
          unitId={unit.id}
          unitName={unit.display_name}
          preview={preview}
          linkRoles={linkRoles}
        />
      )}
      {(offset > 0 || unit.member_count > 20) && (
        <RosterPagination
          offset={offset}
          count={members.data?.length ?? 0}
          total={unit.member_count}
          pending={members.isFetching}
          onPageChange={(nextOffset) => {
            setSearch("");
            setOffset(nextOffset);
          }}
        />
      )}
    </>
  );
}
