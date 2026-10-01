import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { ArrowLeft, Pencil } from "lucide-react";

import { Avatar } from "../components/avatar";
import { EmptyState } from "../components/empty-state";
import { ErrorState } from "../components/error-state";
import { LoadingState } from "../components/loading-state";
import { MemberEditor } from "../components/member-editor";
import { MemberRolePills } from "../components/member-role-pills";
import { PageHeading } from "../components/page-heading";
import { Button } from "../components/ui/button";
import { useWorkspace } from "../components/workspace-context";
import {
  canAssignRanks,
  canAssignRole,
  canManageMemberProfiles,
  canManageRoles,
} from "../lib/permissions";
import { ApiError } from "../lib/session-client";

export function MemberPage({ editing = false }: { editing?: boolean }) {
  const { memberId } = useParams({ strict: false });
  const { unit, source, queryKey } = useWorkspace();
  const member = useQuery({
    queryKey: [...queryKey, "member", memberId],
    queryFn: ({ signal }) => source.member(unit.id, memberId!, signal),
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
  const back = (
    <Button asChild variant="ghost">
      <Link to="/units/$unitId/members" params={{ unitId: unit.id }}>
        <ArrowLeft size={16} aria-hidden="true" /> Members
      </Link>
    </Button>
  );
  if (
    !member.data &&
    member.isError &&
    member.error instanceof ApiError &&
    member.error.status === 404
  )
    return (
      <>
        <PageHeading
          title="Member not found"
          description="This member is not available in this unit."
          action={back}
        />
        <EmptyState title="This member is no longer available">
          Return to Members to see the current roster.
        </EmptyState>
      </>
    );
  if (member.isPending || ranks.isPending || roles.isPending || access.isPending)
    return <LoadingState label="Loading member" />;
  if (!member.data || !ranks.data || !roles.data || !access.data)
    return (
      <ErrorState
        error={member.error ?? ranks.error ?? roles.error ?? access.error}
        retry={() => {
          void member.refetch();
          void ranks.refetch();
          void roles.refetch();
          void access.refetch();
        }}
      />
    );

  const person = member.data;
  const name = person.display_name ?? person.username;
  const rank = ranks.data.find((item) => item.id === person.rank_id);
  const params = { unitId: unit.id, memberId: person.id };
  const editable =
    canAssignRanks(access.data) ||
    canManageMemberProfiles(access.data) ||
    roles.data.some((role) => canAssignRole(access.data, role));

  return (
    <>
      <PageHeading
        title={editing ? "Edit member" : name}
        description={
          editing
            ? `Update ${name}’s details in ${unit.display_name}.`
            : `Member of ${unit.display_name}.`
        }
        action={
          editing ? (
            <Button asChild variant="ghost">
              <Link to="/units/$unitId/members/$memberId" params={params}>
                <ArrowLeft size={16} aria-hidden="true" /> View member
              </Link>
            </Button>
          ) : (
            back
          )
        }
      />
      {(member.isError || ranks.isError || roles.isError || access.isError) && (
        <ErrorState
          error={member.error ?? ranks.error ?? roles.error ?? access.error}
          retry={() => {
            void member.refetch();
            void ranks.refetch();
            void roles.refetch();
            void access.refetch();
          }}
        />
      )}
      <section
        className="role-editor member-page"
        aria-label={editing ? "Edit member details" : "Member details"}
      >
        <div className="member-identity">
          <Avatar name={name} url={person.icon_url} />
          <div>
            <strong>{name}</strong>
            <p>{person.username}</p>
          </div>
        </div>
        {editing ? (
          <MemberEditor
            key={person.id}
            member={person}
            ranks={ranks.data}
            roles={roles.data}
            access={access.data}
          />
        ) : (
          <>
            <dl className="member-details">
              <div>
                <dt>Rank</dt>
                <dd>
                  {rank ? (
                    <Link
                      className="member-profile-link"
                      to="/units/$unitId/ranks/$rankId"
                      params={{ unitId: unit.id, rankId: rank.id }}
                    >
                      {rank.display_name ?? rank.slug}
                    </Link>
                  ) : (
                    "Unspecified rank"
                  )}
                </dd>
              </div>
              <div>
                <dt>Roles</dt>
                <dd>
                  {roles.data.some(
                    (role) => role.kind !== "everyone" && person.role_ids.includes(role.id),
                  ) ? (
                    <MemberRolePills
                      member={person}
                      roles={roles.data}
                      unitId={unit.id}
                      linkRoles={canManageRoles(access.data)}
                    />
                  ) : (
                    "No assigned roles"
                  )}
                </dd>
              </div>
            </dl>
            {editable && (
              <footer className="editor-footer">
                <Button asChild variant="outline">
                  <Link to="/units/$unitId/members/$memberId/edit" params={params}>
                    <Pencil size={16} aria-hidden="true" /> Edit member
                  </Link>
                </Button>
              </footer>
            )}
          </>
        )}
      </section>
    </>
  );
}
