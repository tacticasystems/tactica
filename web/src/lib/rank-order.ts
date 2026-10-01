import { arrayMove } from "@dnd-kit/sortable";
import { canManageRanks } from "./permissions";
import type { Access, Rank } from "./types";

export function moveRank(access: Access, ranks: Rank[], active: string, over: string) {
  if (!canManageRanks(access) || active === over) return null;
  const from = ranks.findIndex((rank) => rank.id === active);
  const to = ranks.findIndex((rank) => rank.id === over);
  if (from < 0 || to < 0) return null;
  return arrayMove(ranks, from, to).map((rank) => rank.id);
}
