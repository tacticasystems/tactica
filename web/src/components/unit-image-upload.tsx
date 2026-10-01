import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Upload } from "lucide-react";

import { deleteUnitFile, uploadUnitImage } from "../lib/api";
import { canManageUnit } from "../lib/permissions";
import { unitsOptions } from "../lib/queries";
import { ApiError } from "../lib/session-client";
import type { Unit } from "../lib/types";
import { FormError } from "./form-error";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
} from "./ui/alert-dialog";
import { useWorkspace } from "./workspace-context";

export function UnitImageUpload({ kind }: { kind: "icon" | "banner" }) {
  const maxMiB = kind === "banner" ? 5 : 2;
  const maxDimension = kind === "banner" ? 4096 : 2048;
  const { unit, source, preview, queryKey } = useWorkspace();
  const queryClient = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const imageUrl = unit[`${kind}_url`];
  const prefix = `/api/v1/units/${unit.id}/${kind}/`;
  const fileId = imageUrl?.startsWith(prefix) ? imageUrl.slice(prefix.length) : null;
  const [validation, setValidation] = useState("");
  const access = useQuery({
    queryKey: [...queryKey, "access"],
    queryFn: ({ signal }) => source.access(unit.id, signal),
    enabled: !preview,
  });
  const upload = useMutation({
    mutationFn: (selected: File) => uploadUnitImage(unit.id, selected, kind),
    onSuccess: async (result) => {
      queryClient.setQueryData<Unit>(queryKey, (current) =>
        current ? { ...current, [`${kind}_url`]: result.url } : current,
      );
      setFile(null);
      if (input.current) input.current.value = "";
      await Promise.all([
        queryClient.invalidateQueries({ queryKey, exact: true }),
        queryClient.invalidateQueries({ queryKey: unitsOptions().queryKey }),
      ]);
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });

  const remove = useMutation({
    mutationFn: (id: string) => deleteUnitFile(unit.id, id),
    onSuccess: async (_, id) => {
      queryClient.setQueryData<Unit>(queryKey, (current) =>
        current && current[`${kind}_url`] === `${prefix}${id}`
          ? { ...current, [`${kind}_url`]: null }
          : current,
      );
      setConfirmRemove(false);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey, exact: true }),
        queryClient.invalidateQueries({ queryKey: unitsOptions().queryKey }),
      ]);
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });
  const busy = upload.isPending || remove.isPending;

  if (preview)
    return (
      <p className="field-help">
        {kind === "icon" ? "Icon" : "Banner"} uploads are available in your own units.
      </p>
    );
  if (access.isPending) return <p role="status">Checking profile permissions…</p>;
  if (access.isError)
    return (
      <div>
        <FormError message={`Could not check ${kind} upload permissions.`} />
        <Button variant="outline" onClick={() => void access.refetch()}>
          Try again
        </Button>
      </div>
    );
  if (!canManageUnit(access.data))
    return <p className="field-help">You need Manage unit permission to change this {kind}.</p>;

  const error =
    validation ||
    (upload.error instanceof ApiError && upload.error.status === 403
      ? `You no longer have permission to change the unit ${kind}.`
      : upload.error instanceof Error
        ? upload.error.message
        : "");

  return (
    <form
      className="unit-icon-upload"
      aria-label={`Upload unit ${kind}`}
      aria-busy={busy}
      onSubmit={(event) => {
        event.preventDefault();
        if (file && !validation && !busy) {
          remove.reset();
          upload.mutate(file);
        }
      }}
    >
      <Label htmlFor={`unit-${kind}-file`}>Unit {kind}</Label>
      <p id={`unit-${kind}-help`} className="field-help">
        PNG, JPEG or WebP. Up to {maxMiB} MiB and {maxDimension} × {maxDimension} pixels. Images are
        resized to fit.
      </p>
      <div className="unit-icon-upload-controls">
        <Input
          ref={input}
          id={`unit-${kind}-file`}
          type="file"
          accept="image/png,image/jpeg,image/webp"
          aria-describedby={`unit-${kind}-help unit-${kind}-feedback`}
          aria-invalid={Boolean(error)}
          disabled={busy}
          onChange={(event) => {
            upload.reset();
            remove.reset();
            const selected = event.currentTarget.files?.[0] ?? null;
            setFile(selected);
            setValidation(
              !selected
                ? ""
                : selected.size === 0
                  ? "This file is empty. Choose an image."
                  : selected.size > maxMiB * 1024 * 1024
                    ? `This image is too large. Choose an image under ${maxMiB} MiB.`
                    : !["image/png", "image/jpeg", "image/webp"].includes(selected.type)
                      ? "Choose a PNG, JPEG or WebP image."
                      : "",
            );
          }}
        />
        <Button type="submit" disabled={!file || Boolean(validation) || busy}>
          <Upload size={16} aria-hidden="true" />
          {upload.isPending ? "Uploading…" : `Upload ${kind}`}
        </Button>
        {fileId && (
          <AlertDialog
            open={confirmRemove}
            onOpenChange={(open) => {
              if (!remove.isPending) {
                setConfirmRemove(open);
                if (open) {
                  remove.reset();
                  upload.reset();
                }
              }
            }}
          >
            <AlertDialogTrigger asChild>
              <Button type="button" variant="outline" disabled={busy}>
                Remove {kind}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Remove unit {kind}?</AlertDialogTitle>
                <AlertDialogDescription>
                  This deletes the current {kind}. You can upload a new image afterward.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <FormError message={remove.error instanceof Error ? remove.error.message : ""} />
              <AlertDialogFooter>
                <AlertDialogCancel disabled={remove.isPending}>Cancel</AlertDialogCancel>
                <Button
                  type="button"
                  variant="destructive"
                  disabled={remove.isPending}
                  onClick={() => remove.mutate(fileId)}
                >
                  {remove.isPending ? "Removing…" : `Remove ${kind}`}
                </Button>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        )}
      </div>
      <div id={`unit-${kind}-feedback`}>
        <FormError message={error} />
        {remove.isSuccess && <p role="status">Unit {kind} removed.</p>}
        {(upload.isPending || upload.isSuccess) && (
          <p role="status" aria-live="polite">
            {upload.isPending
              ? `Uploading your unit ${kind}…`
              : upload.isSuccess
                ? `Unit ${kind} updated.`
                : ""}
          </p>
        )}
      </div>
    </form>
  );
}
