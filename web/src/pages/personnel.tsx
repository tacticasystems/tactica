import { useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { ChevronLeft, ChevronRight, Search } from "lucide-react";
import { useWorkspace } from "../components/workspace";
import { Avatar, EmptyState, ErrorState, LoadingState, PageHeading } from "../components/shared";
import { Button } from "../components/ui/button";
import { safeImage } from "../lib/utils";

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
  const filtered = members.data?.filter((member) =>
    (member.display_name ?? member.username)
      .toLocaleLowerCase()
      .includes(search.trim().toLocaleLowerCase()),
  );
  return (
    <>
      <PageHeading
        title="Personnel"
        description={`The ${unit.display_name} roster.`}
        action={
          <label className="search-input">
            <Search size={17} aria-hidden="true" />
            <span className="sr-only">Search this page</span>
            <input
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
      {members.isPending || ranks.isPending ? (
        <LoadingState label="Loading personnel" />
      ) : members.isError || ranks.isError ? (
        <ErrorState
          error={members.error ?? ranks.error}
          retry={() => {
            void members.refetch();
            void ranks.refetch();
          }}
        />
      ) : filtered?.length === 0 ? (
        <EmptyState title={search ? "No matching members on this page" : "No members on this page"}>
          {search
            ? "Try another name or clear your search."
            : "Choose the previous page if the roster has changed."}
        </EmptyState>
      ) : (
        <div className="table-wrap">
          <table className="roster-table">
            <caption className="sr-only">{unit.display_name} personnel roster</caption>
            <thead>
              <tr>
                <th scope="col">Member</th>
                <th scope="col">Rank</th>
              </tr>
            </thead>
            <tbody>
              {filtered?.map((member) => {
                const name = member.display_name ?? member.username;
                const rank = ranks.data?.find((rank) => rank.id === member.rank_id);
                const icon = safeImage(rank?.icon_url ?? null);
                return (
                  <tr key={member.id}>
                    <td>
                      <div className="member-name">
                        <Avatar name={name} url={member.icon_url} />
                        <strong>{name}</strong>
                        {member.display_name && member.display_name !== member.username && (
                          <span className="member-username">{member.username}</span>
                        )}
                      </div>
                    </td>
                    <td>
                      <div className="rank-name">
                        {icon && <img src={icon} alt="" className="rank-icon" />}
                        <span>{rank?.display_name ?? rank?.slug ?? "Unspecified rank"}</span>
                        {preview && <span className="rank-abbreviation">{rank?.slug}</span>}
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
      {(offset > 0 || unit.member_count > 20) && (
        <div className="pagination">
          <span>
            Showing {offset + (members.data?.length ? 1 : 0)}–{offset + (members.data?.length ?? 0)}{" "}
            of {unit.member_count}
          </span>
          <div>
            <Button
              variant="outline"
              disabled={offset === 0 || members.isFetching}
              onClick={() => {
                setSearch("");
                setOffset(Math.max(0, offset - 20));
              }}
            >
              <ChevronLeft size={16} />
              Previous
            </Button>
            <Button
              variant="outline"
              disabled={
                (members.data?.length ?? 0) < 20 ||
                offset + (members.data?.length ?? 0) >= unit.member_count ||
                members.isFetching
              }
              onClick={() => {
                setSearch("");
                setOffset(offset + 20);
              }}
            >
              Next
              <ChevronRight size={16} />
            </Button>
          </div>
        </div>
      )}
    </>
  );
}
