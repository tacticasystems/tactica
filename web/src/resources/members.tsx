import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { MemberName } from "../components/member-name";
import { MemberRolePills } from "../components/member-role-pills";
import { useWorkspace } from "../components/workspace-context";
import { canManageRoles } from "../lib/permissions";
import type { ResourceDefinition } from "../lib/resource";
import type { Member, Rank, Role } from "../lib/types";
import { safeImage } from "../lib/utils";
import { MemberPage } from "../pages/member";

interface MemberContext {
  ranks: Rank[];
  roles: Role[];
  unitId: string;
  preview: boolean;
  linkRoles: boolean;
}

export const membersResource: ResourceDefinition<Member, MemberContext> = {
  apiBase: "/units/{unit_id}/members",
  itemKey: "member",
  label: "Member",
  pluralLabel: "Members",
  name: (member) => member.display_name ?? member.username,
  searchText: (member) => member.display_name ?? member.username,
  listActions: false,
  fields: [
    {
      id: "member",
      label: "Member",
      in: ["list"],
      read: (member, { ranks, unitId }) => (
        <MemberName
          member={member}
          rank={ranks.find((rank) => rank.id === member.rank_id)}
          unitId={unitId}
        />
      ),
    },
    {
      id: "rank",
      label: "Rank",
      in: ["list", "view"],
      read: (member, { ranks, unitId, preview }) => {
        const rank = ranks.find((rank) => rank.id === member.rank_id);
        const icon = safeImage(rank?.icon_url ?? null);
        return (
          <div className="rank-name">
            {icon && <img src={icon} alt="" className="rank-icon" />}
            {rank ? (
              <Link to="/units/$unitId/ranks/$rankId" params={{ unitId, rankId: rank.id }}>
                {rank.display_name ?? rank.slug}
              </Link>
            ) : (
              <span>Unspecified rank</span>
            )}
            {preview && <span className="rank-abbreviation">{rank?.slug}</span>}
          </div>
        );
      },
    },
    {
      id: "roles",
      label: "Roles",
      in: ["list", "view"],
      read: (member, { roles, unitId, linkRoles }) => (
        <MemberRolePills member={member} roles={roles} unitId={unitId} linkRoles={linkRoles} />
      ),
    },
  ],
  useResource: () => {
    const { unit, source, queryKey, preview } = useWorkspace();
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
    return {
      queryKey,
      source: {
        list: (offset, signal) => source.members(unit.id, offset, signal),
        get: (id, signal) => source.member(unit.id, id, signal),
      },
      context: {
        ranks: ranks.data ?? [],
        roles: roles.data ?? [],
        unitId: unit.id,
        preview,
        linkRoles: !!access.data && canManageRoles(access.data),
      },
      total: unit.member_count,
      description: `The ${unit.display_name} roster.`,
      caption: `${unit.display_name} members roster`,
      pending: ranks.isPending || roles.isPending,
      error: ranks.error ?? roles.error,
      refetch: () => {
        void ranks.refetch();
        void roles.refetch();
        void access.refetch();
      },
    };
  },
  viewView: MemberPage,
  editView: () => <MemberPage editing />,
};
