import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { useState } from "react";
import type { SubmitEvent } from "react";
import { Link, useNavigate } from "@tanstack/react-router";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ArrowRight, Plus } from "lucide-react";
import { createUnit } from "../lib/api";
import { queryClient, unitsOptions, userOptions } from "../lib/queries";
import { AppShell, RequireSession } from "../components/workspace";
import { Avatar, EmptyState, ErrorState, LoadingState, PageHeading } from "../components/shared";
import { Button } from "../components/ui/button";

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
  const navigate = useNavigate();
  const mutation = useMutation({
    mutationFn: ({ name, slug }: { name: string; slug: string }) => createUnit(name, slug),
    onSuccess: async (unit) => {
      await queryClient.invalidateQueries({ queryKey: unitsOptions().queryKey });
      await navigate({ to: "/units/$unitId/personnel", params: { unitId: unit.id } });
    },
  });
  const submit = (event: SubmitEvent<HTMLFormElement>) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      name: String(data.get("name") ?? "").trim(),
      slug: String(data.get("slug") ?? "").trim(),
    });
  };
  return (
    <AppShell label="Your units" accountName={user.data?.display_name ?? user.data?.username}>
      <PageHeading
        title="Your units"
        description="Choose a unit to open its workspace."
        action={
          <Button
            onClick={() => {
              mutation.reset();
              setCreating(!creating);
            }}
            variant="outline"
          >
            <Plus size={16} />
            Create a unit
          </Button>
        }
      />
      {creating && (
        <form className="create-unit-form" onSubmit={submit} aria-busy={mutation.isPending}>
          <h2>Create a unit</h2>
          <div className="form-grid">
            <div>
              <Label htmlFor="unit-name">Unit name</Label>
              <Input
                id="unit-name"
                name="name"
                placeholder="9 Rifles"
                required
                maxLength={100}
                autoFocus
              />
            </div>
            <div>
              <Label htmlFor="unit-slug">Unit handle</Label>
              <Input
                id="unit-slug"
                name="slug"
                placeholder="9-rifles"
                pattern="[a-z0-9]+(-[a-z0-9]+)*"
                required
                maxLength={100}
              />
              <p className="field-help">Lowercase letters, numbers, and hyphens.</p>
            </div>
          </div>
          {mutation.isError && (
            <p role="alert" className="form-error">
              {mutation.error.message}
            </p>
          )}
          <div className="actions">
            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "Creating…" : "Create unit"}
            </Button>
            <Button
              type="button"
              variant="ghost"
              onClick={() => setCreating(false)}
              disabled={mutation.isPending}
            >
              Cancel
            </Button>
          </div>
        </form>
      )}
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
              to="/units/$unitId/personnel"
              params={{ unitId: unit.id }}
              className="unit-list-row"
            >
              <Avatar name={unit.display_name} url={unit.icon_url} />
              <div>
                <h2>{unit.display_name}</h2>
                <p>
                  {unit.member_count} {unit.member_count === 1 ? "member" : "members"}
                </p>
              </div>
              <ArrowRight size={19} />
            </Link>
          ))}
        </div>
      )}
    </AppShell>
  );
}
