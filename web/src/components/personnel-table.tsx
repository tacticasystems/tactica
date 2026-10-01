import { Link } from "@tanstack/react-router";

import type { Member, Rank, Role } from "../lib/types";
import { safeImage } from "../lib/utils";

import { Avatar } from "./avatar";
import { MemberRolePills } from "./member-role-pills";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  TableCaption,
} from "./ui/table";

export function PersonnelTable({
  members,
  ranks,
  roles,
  unitId,
  unitName,
  preview,
  linkRoles,
}: {
  members: Member[];
  ranks: Rank[];
  roles: Role[];
  unitId: string;
  unitName: string;
  preview: boolean;
  linkRoles: boolean;
}) {
  return (
    <div className="table-wrap">
      <Table className="roster-table">
        <TableCaption className="sr-only">{unitName} personnel roster</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead scope="col">Member</TableHead>
            <TableHead scope="col">Rank</TableHead>
            <TableHead scope="col">Roles</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {members.map((member) => {
            const name = member.display_name ?? member.username;
            const rank = ranks.find((rank) => rank.id === member.rank_id);
            const icon = safeImage(rank?.icon_url ?? null);
            return (
              <TableRow key={member.id}>
                <TableCell>
                  <div className="member-name">
                    <Avatar name={name} url={member.icon_url} />
                    <div className="flex flex-col gap-1">
                      <Link
                        className="member-profile-link"
                        to="/units/$unitId/personnel/$memberId"
                        params={{ unitId, memberId: member.id }}
                      >
                        <strong>{rank?.slug}</strong>
                        &nbsp;
                        {name}
                      </Link>
                      {member.display_name && member.display_name !== member.username && (
                        <span className="member-username">{member.username}</span>
                      )}
                    </div>
                  </div>
                </TableCell>
                <TableCell>
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
                </TableCell>
                <TableCell>
                  <MemberRolePills
                    member={member}
                    roles={roles}
                    unitId={unitId}
                    linkRoles={linkRoles}
                  />
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </div>
  );
}
