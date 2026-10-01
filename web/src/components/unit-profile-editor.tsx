import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { useState } from "react";

import { updateUnitProfile } from "../lib/api";
import { canManageUnit } from "../lib/permissions";
import { unitsOptions } from "../lib/queries";
import { ApiError } from "../lib/session-client";
import type { Unit, UnitProfileInput } from "../lib/types";
import { FormError } from "./form-error";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Textarea } from "./ui/textarea";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from "./ui/alert-dialog";
import { useWorkspace } from "./workspace-context";

type Draft = { display_name: string; slug: string; biography: string };
const fields = (unit: Unit): Draft => ({
  display_name: unit.display_name,
  slug: unit.slug,
  biography: unit.biography ?? "",
});

export function UnitProfileEditor() {
  const { unit, source, preview, queryKey } = useWorkspace();
  const queryClient = useQueryClient();
  // A separate draft survives unrelated cache refreshes, including icon uploads.
  const [draft, setDraft] = useState<Partial<Draft> | null>(null);
  const [notice, setNotice] = useState("");
  const saved = fields(unit);
  const values = { ...saved, ...draft };
  const dirty = (Object.keys(saved) as (keyof Draft)[]).some((key) => values[key] !== saved[key]);
  const access = useQuery({
    queryKey: [...queryKey, "access"],
    queryFn: ({ signal }) => source.access(unit.id, signal),
    enabled: !preview,
  });
  const editable = !preview && !!access.data && canManageUnit(access.data);
  const save = useMutation({
    mutationFn: (input: UnitProfileInput) => updateUnitProfile(unit.id, input),
    onSuccess: async (updated) => {
      queryClient.setQueryData(queryKey, updated);
      setDraft(null);
      setNotice("Profile saved.");
      await Promise.all([
        queryClient.invalidateQueries({ queryKey, exact: true }),
        queryClient.invalidateQueries({ queryKey: unitsOptions().queryKey }),
      ]);
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });
  const blocker = useBlocker({
    shouldBlockFn: ({ current, next }) => dirty && current.pathname !== next.pathname,
    enableBeforeUnload: dirty,
    withResolver: true,
  });
  const change = (key: keyof Draft, value: string) => {
    setDraft({ ...draft, [key]: value });
    setNotice("");
    save.reset();
  };
  const reset = () => {
    setDraft(null);
    setNotice("");
    save.reset();
  };
  const error =
    save.error instanceof ApiError && save.error.status === 409
      ? "That unit handle is already in use. Choose another."
      : save.error instanceof ApiError && save.error.status === 403
        ? "You no longer have permission to edit this profile. Your changes have not been saved."
        : save.error instanceof Error
          ? save.error.message
          : "";

  return (
    <>
      <form
        aria-label="Edit unit profile"
        aria-busy={save.isPending}
        onSubmit={(event) => {
          event.preventDefault();
          if (!editable || !dirty || save.isPending) return;
          const input: UnitProfileInput = {};
          if (values.display_name !== saved.display_name)
            input.display_name = values.display_name.trim();
          if (values.slug !== saved.slug) input.slug = values.slug.trim();
          if (values.biography !== saved.biography)
            input.biography = values.biography.trim() || null;
          save.mutate(input);
        }}
      >
        {preview ? (
          <p className="field-help">Profile editing is available in your own units.</p>
        ) : access.isPending ? (
          <p role="status">Checking profile permissions…</p>
        ) : access.isError ? (
          <div>
            <FormError message="Could not check profile editing permissions." />
            <Button type="button" variant="outline" onClick={() => void access.refetch()}>
              Try again
            </Button>
          </div>
        ) : (
          !editable && (
            <p className="field-help">You need Manage unit permission to edit this profile.</p>
          )
        )}
        <div className="unit-profile-field">
          <Label htmlFor="unit-name">Display name</Label>
          <Input
            id="unit-name"
            value={values.display_name}
            onChange={(e) => change("display_name", e.target.value)}
            readOnly={!editable}
            disabled={save.isPending}
            required
            maxLength={100}
          />
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-slug">Unit handle</Label>
          <Input
            id="unit-slug"
            value={values.slug}
            onChange={(e) => change("slug", e.target.value)}
            readOnly={!editable}
            disabled={save.isPending}
            required
            maxLength={100}
            pattern="[a-z0-9]+(-[a-z0-9]+)*"
            aria-describedby="unit-handle-help"
          />
          <p id="unit-handle-help" className="field-help">
            Lowercase letters, numbers, and single hyphens.
          </p>
        </div>
        <div className="unit-profile-field">
          <Label htmlFor="unit-biography">Biography</Label>
          <Textarea
            id="unit-biography"
            value={values.biography}
            onChange={(e) => change("biography", e.target.value)}
            placeholder="Tell people about your unit"
            readOnly={!editable}
            disabled={save.isPending}
            rows={5}
            maxLength={5000}
            aria-describedby="unit-biography-help"
          />
          <p id="unit-biography-help" className="field-help">
            {values.biography.length.toLocaleString()} / 5,000 characters
          </p>
        </div>
        <FormError message={error} />
        {notice && <p role="status">{notice}</p>}
        {(editable || dirty) && (
          <footer className="editor-footer">
            <span className="editor-status">{dirty ? "Unsaved changes" : "Up to date"}</span>
            <div className="actions">
              <Button
                type="button"
                variant="ghost"
                onClick={reset}
                disabled={!dirty || save.isPending}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={!editable || !dirty || save.isPending || !values.display_name.trim()}
              >
                {save.isPending ? "Saving…" : "Save changes"}
              </Button>
            </div>
          </footer>
        )}
      </form>
      <AlertDialog
        open={blocker.status === "blocked"}
        onOpenChange={(open) => {
          if (!open && blocker.status === "blocked") blocker.reset();
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Discard profile changes?</AlertDialogTitle>
            <AlertDialogDescription>
              Your edits have not been saved. Stay here to keep editing, or discard them to leave.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Keep editing</AlertDialogCancel>
            <AlertDialogAction
              disabled={save.isPending}
              onClick={() => {
                reset();
                if (blocker.status === "blocked") blocker.proceed();
              }}
            >
              Discard changes
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
