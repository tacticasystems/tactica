export { cn } from "cn";

export function initials(name: string) {
  return name
    .trim()
    .split(/[\s.]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => word[0])
    .join("")
    .toUpperCase();
}

export function safeImage(url: string | null) {
  if (!url) return undefined;
  try {
    const parsed = new URL(url, window.location.origin);
    if (
      parsed.protocol === "https:" ||
      (parsed.protocol === "http:" && parsed.origin === window.location.origin)
    )
      return parsed.href;
  } catch {
    /* Invalid optional artwork uses the initials fallback. */
  }
  return undefined;
}
