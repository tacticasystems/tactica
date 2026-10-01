import { UnitProfileEditor } from "../components/unit-profile-editor";

import { UnitImageUpload } from "../components/unit-image-upload";

import { safeImage } from "../lib/utils";

import { PageHeading } from "../components/page-heading";
import { UnitIdentity } from "../components/unit-identity";
import { useWorkspace } from "../components/workspace-context";

export function ProfilePage() {
  const { unit } = useWorkspace();
  const banner = safeImage(unit.banner_url);

  return (
    <>
      <PageHeading title="Profile" description="Your unit’s public profile and identity." />
      <section className="unit-profile-fields" aria-label="Unit profile">
        {banner && <img src={banner} alt="Unit banner" className="unit-banner" />}
        <div className="unit-profile">
          <UnitIdentity unit={unit} />
        </div>
        <UnitImageUpload kind="icon" />
        <UnitImageUpload kind="banner" />
        <UnitProfileEditor />
      </section>
    </>
  );
}
