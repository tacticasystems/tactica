import type { Unit } from "../lib/types";

import { Avatar } from "./avatar";

export function UnitIdentity({ unit, showSlug = false }: { unit: Unit; showSlug?: boolean }) {
  return (
    <>
      <Avatar name={unit.display_name} url={unit.icon_url} />
      <div>
        <h2>{unit.display_name}</h2>
        <p>
          {unit.member_count} {unit.member_count === 1 ? "member" : "members"}
          {showSlug && ` · ${unit.slug}`}
        </p>
      </div>
    </>
  );
}
