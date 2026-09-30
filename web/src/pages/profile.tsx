import { useWorkspace } from "../components/workspace";
import { Avatar, PageHeading } from "../components/shared";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Textarea } from "../components/ui/textarea";
import { safeImage } from "../lib/utils";

export function ProfilePage() {
  const { unit } = useWorkspace();
  const banner = safeImage(unit.banner_url);
  return (
    <>
      <PageHeading
        title="Profile"
        description="Your unit’s profile. These fields are read-only for now."
      />
      <section className="unit-profile-fields" aria-label="Unit profile">
        {banner && <img src={banner} alt="Unit banner" className="unit-banner" />}
        <div className="unit-profile">
          <Avatar name={unit.display_name} url={unit.icon_url} />
          <div>
            <h2>{unit.display_name}</h2>
            <p>
              {unit.member_count} {unit.member_count === 1 ? "member" : "members"}
            </p>
          </div>
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-name">Display name</Label>
          <Input id="unit-name" value={unit.display_name} readOnly />
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-slug">Slug</Label>
          <Input id="unit-slug" value={unit.slug} readOnly />
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-biography">Biography</Label>
          <Textarea
            id="unit-biography"
            value={unit.biography ?? ""}
            placeholder="No biography added"
            readOnly
            rows={5}
          />
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-icon">Icon URL</Label>
          <Input id="unit-icon" value={unit.icon_url ?? ""} placeholder="No icon added" readOnly />
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-banner">Banner URL</Label>
          <Input
            id="unit-banner"
            value={unit.banner_url ?? ""}
            placeholder="No banner added"
            readOnly
          />
        </div>
      </section>
    </>
  );
}
