import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { useWorkspace } from "./workspace";
import { Avatar, LoadingState } from "./shared";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { canAssignRole } from "../lib/permissions";
import { errorMessage, queryClient } from "../lib/queries";
import type { Access, Member, Role } from "../lib/types";

export function RoleMembers({
  role,
  access,
  pending,
}: {
  role: Role;
  access: Access;
  pending: boolean;
}) {
  const { unit, source, queryKey, preview } = useWorkspace();
  const [adding, setAdding] = useState(false);
  const [search, setSearch] = useState("");
  const [notice, setNotice] = useState("");
  const roster = useQuery({
    queryKey: [...queryKey, "members", "all"],
    queryFn: ({ signal }) => source.allMembers(unit.id, signal),
  });
  const bindingKey = [...queryKey, "role-members", role.id];
  const bindings = useQuery({
    queryKey: bindingKey,
    queryFn: ({ signal }) => source.roleMembers(unit.id, role.id, signal),
  });
  const change = useMutation({
    mutationFn: ({ member, assigned }: { member: Member; assigned: boolean }) =>
      source.setRoleMember(unit.id, role.id, member.id, assigned),
    onSuccess: async (_result, { member, assigned }) => {
      queryClient.setQueryData<string[]>(bindingKey, (ids) =>
        assigned
          ? [...new Set([...(ids ?? []), member.id])]
          : ids?.filter((id) => id !== member.id),
      );
      setNotice(
        `${member.display_name ?? member.username} ${assigned ? "added to" : "removed from"} ${role.display_name}.`,
      );
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: bindingKey }),
        queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] }),
        queryClient.invalidateQueries({ queryKey: [...queryKey, "members"] }),
      ]);
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: bindingKey });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "members"] });
    },
  });
  const manageable = canAssignRole(access, role);
  const busy = pending || change.isPending || bindings.isFetching;
  const ids = new Set(bindings.data);
  const assigned = roster.data?.filter((member) => ids.has(member.id)) ?? [];
  const candidates = roster.data?.filter((member) => !ids.has(member.id)) ?? [];
  const list = adding && manageable ? candidates : assigned;
  const filtered = list.filter((member) =>
    `${member.display_name ?? ""} ${member.username}`
      .toLocaleLowerCase()
      .includes(search.trim().toLocaleLowerCase()),
  );
  return (
    <section
      className="role-editor role-members"
      aria-label={`${role.display_name} members`}
      aria-busy={change.isPending}
    >
      <header className="editor-heading">
        <div>
          <h2>{role.display_name} members</h2>
          <p>
            {bindings.isSuccess && roster.isSuccess
              ? `${assigned.length} ${assigned.length === 1 ? "member has" : "members have"} this role.`
              : "See who has this role."}
          </p>
        </div>
        {manageable && (
          <Button
            type="button"
            variant="outline"
            disabled={busy}
            aria-pressed={adding}
            onClick={() => {
              setAdding(!adding);
              setSearch("");
              change.reset();
              setNotice("");
            }}
          >
            {!adding && <Plus size={16} aria-hidden="true" />}
            {adding ? "Done adding" : "Add members"}
          </Button>
        )}
      </header>
      {preview && <p className="access-note">Assignments here are examples for trying the UI.</p>}
      {role.kind === "everyone" ? (
        <p className="access-note">
          Everyone applies automatically to all current and future members. People cannot be added
          or removed individually.
        </p>
      ) : (
        !manageable && (
          <p className="access-note">
            You can view these members. Assigning or removing this role requires Assign roles and a
            higher role position, or unit ownership.
          </p>
        )
      )}
      {notice && (
        <p className="save-notice" role="status">
          {notice}
        </p>
      )}
      {change.isError && (
        <p className="form-error" role="alert">
          {errorMessage(change.error)} Try again after permissions have refreshed.
        </p>
      )}
      {roster.isPending || bindings.isPending ? (
        <LoadingState label="Loading role members" />
      ) : roster.isError || bindings.isError ? (
        <div className="form-error" role="alert">
          <p>{errorMessage(roster.error ?? bindings.error)}</p>
          <Button
            type="button"
            variant="outline"
            onClick={() => {
              void roster.refetch();
              void bindings.refetch();
            }}
          >
            Try again
          </Button>
        </div>
      ) : (
        <>
          <label className="role-member-search">
            <span>
              {adding && manageable
                ? "Find members to add"
                : role.kind === "everyone"
                  ? "Search members"
                  : "Search assigned members"}
            </span>
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
                : adding && manageable
                  ? "Every unit member already has this role."
                  : "No members have this role yet."}
            </p>
          ) : (
            <ul className="role-member-list">
              {filtered.map((member) => {
                const name = member.display_name ?? member.username;
                const assigned = ids.has(member.id);
                const changing = change.isPending && change.variables.member.id === member.id;
                return (
                  <li key={member.id}>
                    <div className="role-member-identity">
                      <Avatar name={name} url={member.icon_url} size="small" />
                      <div>
                        <strong>{name}</strong>
                        <small>{member.username}</small>
                      </div>
                    </div>
                    {manageable && (
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        disabled={busy}
                        aria-label={`${assigned ? "Remove" : "Add"} ${name} ${assigned ? "from" : "to"} ${role.display_name}`}
                        onClick={() => {
                          setNotice("");
                          change.mutate({ member, assigned: !assigned });
                        }}
                      >
                        {changing
                          ? assigned
                            ? "Removing…"
                            : "Adding…"
                          : assigned
                            ? "Remove"
                            : "Add"}
                      </Button>
                    )}
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
