import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { Pencil } from "lucide-react";
import { useWorkspace } from "../components/workspace-context";
import { PageHeading } from "../components/page-heading";
import { RankMembers } from "../components/rank-details";
import { LoadingState } from "../components/loading-state";
import { ErrorState } from "../components/error-state";
import { EmptyState } from "../components/empty-state";
import { Button } from "../components/ui/button";
import { canManageRanks } from "../lib/permissions";
import { safeImage } from "../lib/utils";

export function RankPage() {
  const { rankId } = useParams({ from: "/units/$unitId/ranks/$rankId" });
  const { unit, source, queryKey } = useWorkspace();
  const ranks = useQuery({
    queryKey: [...queryKey, "ranks"],
    queryFn: ({ signal }) => source.ranks(unit.id, signal),
  });
  const access = useQuery({
    queryKey: [...queryKey, "access"],
    queryFn: ({ signal }) => source.access(unit.id, signal),
  });
  if (ranks.isPending) return <LoadingState label="Loading rank" />;
  if (ranks.isError) return <ErrorState error={ranks.error} retry={() => void ranks.refetch()} />;
  const rank = ranks.data.find((item) => item.id === rankId);
  if (!rank)
    return (
      <>
        <PageHeading
          title="Rank not found"
          description="This rank is not available in this unit."
        />
        <EmptyState title="This rank is no longer available">
          Return to Ranks to see the current rank structure.
        </EmptyState>
      </>
    );
  const icon = safeImage(rank.icon_url);
  return (
    <>
      <PageHeading
        title={rank.display_name ?? rank.slug}
        description={rank.description?.trim() ? rank.description : `Rank in ${unit.display_name}.`}
        action={
          access.data &&
          canManageRanks(access.data) && (
            <Button asChild variant="outline">
              <Link
                to="/units/$unitId/admin/ranks"
                params={{ unitId: unit.id }}
                search={{ rankId: rank.id }}
              >
                <Pencil size={16} aria-hidden="true" /> Edit rank
              </Link>
            </Button>
          )
        }
      />
      <div className="rank-view">
        <section className="rank-summary" aria-label="Rank details">
          <header className="editor-heading">
            <div className="rank-name">
              {icon && <img src={icon} className="rank-icon" alt="" />}
              <h2>Rank details</h2>
            </div>
          </header>
          <dl className="member-details">
            <div>
              <dt>Abbreviation</dt>
              <dd>{rank.slug}</dd>
            </div>
          </dl>
          {access.isError && (
            <ErrorState error={access.error} retry={() => void access.refetch()} />
          )}
        </section>
        <RankMembers key={rank.id} rank={rank} />
      </div>
    </>
  );
}
