import { Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";

import { safeImage } from "../lib/utils";

import { PageHeading } from "../components/page-heading";
import { UnitIdentity } from "../components/unit-identity";
import { useWorkspace } from "../components/workspace-context";

export function OverviewPage() {
  const { unit } = useWorkspace();
  const banner = safeImage(unit.banner_url);

  return (
    <>
      <PageHeading title="Unit overview" description="Your unit, at a glance." />
      {banner && <img src={banner} alt="" className="unit-banner" />}
      <section className="unit-profile">
        <UnitIdentity unit={unit} showSlug />
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
