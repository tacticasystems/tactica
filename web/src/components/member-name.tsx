import { Link } from "@tanstack/react-router";
import type { Member, Rank } from "../lib/types";
import { Avatar } from "./avatar";

export function MemberName({
  member,
  rank,
  unitId,
}: {
  member: Member;
  rank?: Rank;
  unitId: string;
}) {
  const name = member.display_name ?? member.username;
  return (
    <div className="member-name">
      <Avatar name={name} url={member.icon_url} />
      <div className="flex flex-col gap-1">
        <Link
          className="member-profile-link"
          to="/units/$unitId/members/$memberId"
          params={{ unitId, memberId: member.id }}
        >
          <strong>{rank?.slug}</strong>&nbsp;{name}
        </Link>
        {member.display_name && member.display_name !== member.username && (
          <span className="member-username">{member.username}</span>
        )}
      </div>
    </div>
  );
}
