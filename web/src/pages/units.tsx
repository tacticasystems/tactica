import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowRight, Plus } from "lucide-react";
import { useState } from "react";

import { unitsOptions, userOptions } from "../lib/queries";

import { AppShell } from "../components/app-shell";
import { CreateUnitForm } from "../components/create-unit-form";
import { EmptyState } from "../components/empty-state";
import { ErrorState } from "../components/error-state";
import { LoadingState } from "../components/loading-state";
import { PageHeading } from "../components/page-heading";
import { RequireSession } from "../components/require-session";
import { Button } from "../components/ui/button";
import { UnitIdentity } from "../components/unit-identity";

export function UnitsPage() {
  return (
    <RequireSession>
      <UnitsContent />
    </RequireSession>
  );
}

function UnitsContent() {
  const units = useQuery(unitsOptions());
  const user = useQuery(userOptions());
  const [creating, setCreating] = useState(false);

  return (
    <AppShell label="Your units" accountName={user.data?.display_name ?? user.data?.username}>
      <PageHeading
        title="Your units"
        description="Choose a unit to open its workspace."
        action={
          <Button
            onClick={() => {
              setCreating(!creating);
            }}
            variant="outline"
          >
            <Plus size={16} />
            Create a unit
          </Button>
        }
      />
      {creating && <CreateUnitForm onCancel={() => setCreating(false)} />}
      {units.isPending ? (
        <LoadingState label="Loading your units" />
      ) : units.isError ? (
        <ErrorState error={units.error} retry={() => void units.refetch()} />
      ) : units.data.length === 0 ? (
        <EmptyState title="Your first unit awaits">
          Create a unit to set up its roster, ranks, and roles. Units you belong to will appear
          here.
        </EmptyState>
      ) : (
        <div className="unit-list">
          {units.data.map((unit) => (
            <Link
              key={unit.id}
              to="/units/$unitId"
              params={{ unitId: unit.id }}
              className="unit-list-row"
            >
              <UnitIdentity unit={unit} />
              <ArrowRight size={19} />
            </Link>
          ))}
        </div>
      )}
    </AppShell>
  );
}
