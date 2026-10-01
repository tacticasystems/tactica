import { initials, safeImage } from "../lib/utils";

import { Avatar as AvatarRoot, AvatarImage, AvatarFallback } from "./ui/avatar";

export function Avatar({
  name,
  url,
  size = "normal",
}: {
  name: string;
  url?: string | null;
  size?: "normal" | "small";
}) {
  const src = safeImage(url ?? null);

  return (
    <AvatarRoot className={`avatar avatar-${size}`} aria-hidden="true">
      <AvatarImage src={src} alt="" />
      <AvatarFallback>{initials(name)}</AvatarFallback>
    </AvatarRoot>
  );
}
