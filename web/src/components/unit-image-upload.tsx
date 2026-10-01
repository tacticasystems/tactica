import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Upload } from "lucide-react";

import { uploadUnitImage } from "../lib/api";
import { canManageUnit } from "../lib/permissions";
import { unitsOptions } from "../lib/queries";
import { ApiError } from "../lib/session-client";
import type { Unit } from "../lib/types";
import { FormError } from "./form-error";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { useWorkspace } from "./workspace-context";

export function UnitImageUpload({ kind }: { kind: "icon" | "banner" }) {
  const maxMiB = kind === "banner" ? 5 : 2;
  const maxDimension = kind === "banner" ? 4096 : 2048;
  const { unit, source, preview, queryKey } = useWorkspace();
  const queryClient = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
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
      aria-busy={upload.isPending}
      onSubmit={(event) => {
        event.preventDefault();
        if (file && !validation && !upload.isPending) upload.mutate(file);
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
          disabled={upload.isPending}
          onChange={(event) => {
            upload.reset();
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
        <Button type="submit" disabled={!file || Boolean(validation) || upload.isPending}>
          <Upload size={16} aria-hidden="true" />
          {upload.isPending ? "Uploading…" : `Upload ${kind}`}
        </Button>
      </div>
      <div id={`unit-${kind}-feedback`}>
        <FormError message={error} />
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
