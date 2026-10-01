import { Link } from "@tanstack/react-router";
import { useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";

import type { Rank } from "../lib/types";
import { Avatar } from "./avatar";
import { ErrorState } from "./error-state";
import { LoadingState } from "./loading-state";
import { Input } from "./ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { useWorkspace } from "./workspace-context";

export function RankDetails({ rank, children }: { rank?: Rank; children: ReactNode }) {
  const [tab, setTab] = useState("details");
  const activeTab = rank ? tab : "details";

  return (
    <Tabs className="role-details" value={activeTab} onValueChange={setTab}>
      {rank && (
        <TabsList className="role-view-switch" aria-label="Rank details">
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="members">Members</TabsTrigger>
        </TabsList>
      )}
      {/* Preserve the draft and navigation guard while viewing members. */}
      <TabsContent value="details" forceMount hidden={activeTab !== "details"}>
        {children}
      </TabsContent>
      {rank && (
        <TabsContent value="members">
          <RankMembers key={rank.id} rank={rank} />
        </TabsContent>
      )}
    </Tabs>
  );
}

function RankMembers({ rank }: { rank: Rank }) {
  const { unit, source, queryKey } = useWorkspace();
  const [search, setSearch] = useState("");
  const roster = useQuery({
    queryKey: [...queryKey, "members", "all"],
    queryFn: ({ signal }) => source.allMembers(unit.id, signal),
  });
  const name = rank.display_name ?? rank.slug;
  const assigned = roster.data?.filter((member) => member.rank_id === rank.id) ?? [];
  const filtered = assigned.filter((member) =>
    `${member.display_name ?? ""} ${member.username}`
      .toLocaleLowerCase()
      .includes(search.trim().toLocaleLowerCase()),
  );

  return (
    <section className="role-editor role-members" aria-label={`${name} members`}>
      <header className="editor-heading">
        <div>
          <h2>{name} members</h2>
          <p>
            {roster.isSuccess
              ? `${assigned.length} ${assigned.length === 1 ? "member has" : "members have"} this rank.`
              : "See who has this rank."}
          </p>
        </div>
      </header>
      {roster.isPending ? (
        <LoadingState label="Loading rank members" />
      ) : roster.isError ? (
        <ErrorState error={roster.error} retry={() => void roster.refetch()} />
      ) : (
        <>
          <label className="role-member-search">
            <span>Search assigned members</span>
            <Input
              type="search"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search by name or username"
            />
          </label>
          {filtered.length === 0 ? (
            <p className="role-member-empty">
              {search.trim()
                ? "No matching members. Try another name or username."
                : "No members have this rank yet."}
            </p>
          ) : (
            <ul className="role-member-list">
              {filtered.map((member) => {
                const memberName = member.display_name ?? member.username;
                return (
                  <li key={member.id}>
                    <div className="role-member-identity">
                      <Avatar name={memberName} url={member.icon_url} size="small" />
                      <div>
                        <Link
                          className="member-profile-link"
                          to="/units/$unitId/personnel/$memberId"
                          params={{ unitId: unit.id, memberId: member.id }}
                        >
                          {memberName}
                        </Link>
                        <small>{member.username}</small>
                      </div>
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </>
      )}
    </section>
  );
}
