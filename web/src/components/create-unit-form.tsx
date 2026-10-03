import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { SubmitEvent } from "react";

import { createUnit } from "../lib/api";
import { queryClient, unitsOptions } from "../lib/queries";

import { FormError } from "./form-error";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";

export function CreateUnitForm({ onCancel }: { onCancel: () => void }) {
  const navigate = useNavigate();
  const mutation = useMutation({
    mutationFn: ({ name, slug }: { name: string; slug: string }) => createUnit(name, slug),
    onSuccess: async (unit) => {
      await queryClient.invalidateQueries({ queryKey: unitsOptions().queryKey });
      await navigate({ to: "/units/$unitId/members", params: { unitId: unit.id } });
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
      <FormError message={mutation.isError ? mutation.error.message : ""} />
      <div className="actions">
        <Button type="submit" disabled={mutation.isPending}>
          {mutation.isPending ? "Creating…" : "Create unit"}
        </Button>
        <Button type="button" variant="ghost" onClick={onCancel} disabled={mutation.isPending}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
