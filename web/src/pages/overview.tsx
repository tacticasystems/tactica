import { Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import { useWorkspace } from "../components/workspace";
import { Avatar, PageHeading } from "../components/shared";
import { safeImage } from "../lib/utils";

export function OverviewPage() {
  const { unit } = useWorkspace();
  const banner = safeImage(unit.banner_url);
  return (
    <>
      <PageHeading title="Unit overview" description="Your unit, at a glance." />
      {banner && <img src={banner} alt="" className="unit-banner" />}
      <section className="unit-profile">
        <Avatar name={unit.display_name} url={unit.icon_url} />
        <div>
          <h2>{unit.display_name}</h2>
          <p>
            {unit.member_count} {unit.member_count === 1 ? "member" : "members"} · {unit.slug}
          </p>
        </div>
      </section>
      {unit.biography && <p className="unit-biography">{unit.biography}</p>}
      <div className="overview-links">
        <Link to="/units/$unitId/personnel" params={{ unitId: unit.id }}>
          <div>
            <h2>Personnel</h2>
            <p>View the unit roster and organizational ranks.</p>
          </div>
          <ArrowRight />
        </Link>
        <Link to="/units/$unitId/roles" params={{ unitId: unit.id }}>
          <div>
            <h2>Roles</h2>
            <p>Review roles and the permissions they grant.</p>
          </div>
          <ArrowRight />
        </Link>
      </div>
    </>
  );
}
