import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Search } from "lucide-react";
import { useWorkspace } from "../components/workspace-context";
import { PageHeading } from "../components/page-heading";
import { LoadingState } from "../components/loading-state";
import { ErrorState } from "../components/error-state";
import { EmptyState } from "../components/empty-state";
import { Input } from "../components/ui/input";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/ui/table";
import { safeImage } from "../lib/utils";

export function RanksPage() {
  const { unit, source, queryKey } = useWorkspace();
  const [search, setSearch] = useState("");
  const ranks = useQuery({
    queryKey: [...queryKey, "ranks"],
    queryFn: ({ signal }) => source.ranks(unit.id, signal),
  });
  const filtered =
    ranks.data?.filter((rank) =>
      `${rank.display_name ?? ""} ${rank.slug} ${rank.description ?? ""}`
        .toLocaleLowerCase()
        .includes(search.trim().toLocaleLowerCase()),
    ) ?? [];
  return (
    <>
      <PageHeading
        title="Ranks"
        description={`The ${unit.display_name} rank structure, ordered highest first.`}
        action={
          <label className="search-input">
            <Search size={17} aria-hidden="true" />
            <span className="sr-only">Search ranks</span>
            <Input
              type="search"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search ranks"
            />
          </label>
        }
      />
      <div className="roster-meta">
        <span>
          {ranks.data?.length ?? 0} {ranks.data?.length === 1 ? "rank" : "ranks"}
        </span>
        {search && <span>{filtered.length} match</span>}
        {ranks.isFetching && !ranks.isPending && <span role="status">Updating…</span>}
      </div>
      {ranks.isPending ? (
        <LoadingState label="Loading ranks" />
      ) : ranks.isError ? (
        <ErrorState error={ranks.error} retry={() => void ranks.refetch()} />
      ) : filtered.length === 0 ? (
        <EmptyState title={search ? "No matching ranks" : "No ranks yet"}>
          {search
            ? "Try another name or clear your search."
            : "Ranks will appear here once a rank manager creates them."}
        </EmptyState>
      ) : (
        <div className="table-wrap">
          <Table className="roster-table rank-directory">
            <TableCaption className="sr-only">{unit.display_name} ranks</TableCaption>
            <TableHeader>
              <TableRow>
                <TableHead scope="col">Rank</TableHead>
                <TableHead scope="col">Abbreviation</TableHead>
                <TableHead scope="col">Description</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((rank) => {
                const icon = safeImage(rank.icon_url);
                return (
                  <TableRow key={rank.id}>
                    <TableCell>
                      <div className="rank-name">
                        {icon && <img src={icon} className="rank-icon" alt="" />}
                        <Link
                          className="member-profile-link"
                          to="/units/$unitId/ranks/$rankId"
                          params={{ unitId: unit.id, rankId: rank.id }}
                        >
                          {rank.display_name ?? rank.slug}
                        </Link>
                      </div>
                    </TableCell>
                    <TableCell>{rank.slug}</TableCell>
                    <TableCell className="rank-directory-description">
                      {rank.description ?? "—"}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </>
  );
}
