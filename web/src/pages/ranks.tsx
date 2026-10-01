import { useState } from "react";
import type { FormEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useBlocker } from "@tanstack/react-router";
import { LockKeyhole, Plus, Trash2 } from "lucide-react";
import { useWorkspace } from "../components/workspace-context";
import { EmptyState } from "../components/empty-state";
import { ErrorState } from "../components/error-state";
import { LoadingState } from "../components/loading-state";
import { PageHeading } from "../components/page-heading";
import { RankList } from "../components/rank-list";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Textarea } from "../components/ui/textarea";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../components/ui/alert-dialog";
import { canManageRanks } from "../lib/permissions";
import { errorMessage, queryClient } from "../lib/queries";
import type { Rank, RankInput } from "../lib/types";

export function RanksPage() {
  const { unit, source, queryKey } = useWorkspace();
  const ranksKey = [...queryKey, "ranks"];
  const accessKey = [...queryKey, "access"];
  const ranks = useQuery({
    queryKey: ranksKey,
    queryFn: ({ signal }) => source.ranks(unit.id, signal),
  });
  const access = useQuery({
    queryKey: accessKey,
    queryFn: ({ signal }) => source.access(unit.id, signal),
  });
  const [selected, setSelected] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [dirty, setDirty] = useState(false);
  const [switchTarget, setSwitchTarget] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<Rank | null>(null);
  const [notice, setNotice] = useState("");
  const current =
    selected === "new"
      ? "new"
      : (ranks.data?.find((rank) => rank.id === selected)?.id ?? ranks.data?.[0]?.id);
  const rank = ranks.data?.find((item) => item.id === current);
  const blocker = useBlocker({
    shouldBlockFn: () => dirty,
    enableBeforeUnload: dirty,
    withResolver: true,
  });
  const refreshAccess = () => {
    void queryClient.invalidateQueries({ queryKey: accessKey });
  };
  const save = useMutation({
    mutationFn: ({ id, input }: { id: string | null; input: RankInput }) =>
      source.saveRank(unit.id, id, input),
    onSuccess: async (saved) => {
      setDirty(false);
      setSelected(saved.id);
      await queryClient.invalidateQueries({ queryKey: ranksKey });
      setRevision((value) => value + 1);
      setNotice("Rank saved.");
    },
    onError: () => {
      refreshAccess();
      void queryClient.invalidateQueries({ queryKey: ranksKey });
    },
  });
  const remove = useMutation({
    mutationFn: (target: Rank) => source.deleteRank(unit.id, target.id),
    onSuccess: async () => {
      setDeleteTarget(null);
      setSelected(null);
      setDirty(false);
      await queryClient.invalidateQueries({ queryKey: ranksKey });
      setNotice("Rank deleted.");
    },
    onError: () => {
      refreshAccess();
      void queryClient.invalidateQueries({ queryKey: ranksKey });
    },
  });
  const reorder = useMutation({
    mutationFn: (ids: string[]) => source.reorderRanks(unit.id, ids),
    onSuccess: (updated) => {
      queryClient.setQueryData(ranksKey, updated);
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: ranksKey });
      refreshAccess();
    },
  });
  const pending = save.isPending || remove.isPending || reorder.isPending;
  const select = (id: string) => {
    if (id === current) return;
    if (dirty) {
      setSwitchTarget(id);
      return;
    }
    setSelected(id);
    save.reset();
    setNotice("");
  };
  if (ranks.isPending || access.isPending)
    return (
      <>
        <PageHeading title="Ranks" description="Your unit’s rank structure." />
        <LoadingState label="Loading ranks and permissions" />
      </>
    );
  if (ranks.isError || access.isError)
    return (
      <ErrorState
        error={ranks.error ?? access.error}
        retry={() => {
          void ranks.refetch();
          void access.refetch();
        }}
      />
    );
  const editable = canManageRanks(access.data);
  return (
    <>
      <PageHeading
        title="Ranks"
        description="Your unit’s rank structure, ordered highest first."
        action={
          editable && (
            <Button variant="outline" disabled={pending} onClick={() => select("new")}>
              <Plus size={16} />
              New rank
            </Button>
          )
        }
      />
      {notice && (
        <p className="save-notice" role="status">
          {notice}
        </p>
      )}
      {(switchTarget || blocker.status === "blocked") && (
        <div className="unsaved-prompt" role="alert">
          <p>You have unsaved changes. Discard them to continue?</p>
          <div className="actions">
            <Button
              variant="outline"
              disabled={pending}
              onClick={() => {
                setDirty(false);
                save.reset();
                if (blocker.status === "blocked") blocker.proceed();
                else setSelected(switchTarget);
                setSwitchTarget(null);
              }}
            >
              Discard changes
            </Button>
            <Button
              variant="ghost"
              onClick={() => {
                if (blocker.status === "blocked") blocker.reset();
                setSwitchTarget(null);
              }}
            >
              Keep editing
            </Button>
          </div>
        </div>
      )}
      <div className="roles-layout">
        <RankList
          ranks={ranks.data}
          access={access.data}
          selected={current}
          pending={pending}
          reorderStatus={
            reorder.isPending
              ? "Saving order…"
              : reorder.isError
                ? errorMessage(reorder.error, "rank")
                : reorder.isSuccess
                  ? "Order saved."
                  : ""
          }
          reorderError={reorder.isError}
          onSelect={select}
          onReorder={(ids) => {
            setSelected(current ?? null);
            reorder.mutate(ids);
          }}
        />
        {rank || current === "new" ? (
          <RankEditor
            key={`${current}-${revision}`}
            rank={rank}
            editable={editable}
            pending={pending}
            error={save.isError ? errorMessage(save.error, "rank") : ""}
            onDirty={setDirty}
            onSave={(input) => {
              setNotice("");
              save.mutate({ id: rank?.id ?? null, input });
            }}
            onCancel={() => {
              setDirty(false);
              setSelected(null);
              save.reset();
            }}
            onDelete={
              rank && editable
                ? () => {
                    remove.reset();
                    setDeleteTarget(rank);
                  }
                : undefined
            }
          />
        ) : (
          <EmptyState title="No ranks yet">
            Members can view ranks here once a rank manager creates them.
          </EmptyState>
        )}
      </div>
      <AlertDialog
        open={!!deleteTarget}
        onOpenChange={(open) => {
          if (!open && !remove.isPending) setDeleteTarget(null);
        }}
      >
        <AlertDialogContent aria-busy={remove.isPending}>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete rank?</AlertDialogTitle>
            <AlertDialogDescription>
              Delete “{deleteTarget?.display_name ?? deleteTarget?.slug}”? This cannot be undone.
              Ranks assigned to members or used as the initial rank cannot be deleted.
            </AlertDialogDescription>
          </AlertDialogHeader>
          {remove.isError && (
            <p className="form-error" role="alert">
              {errorMessage(remove.error, "rank")}
            </p>
          )}
          <AlertDialogFooter>
            <AlertDialogCancel disabled={remove.isPending}>Cancel</AlertDialogCancel>
            <Button
              variant="destructive"
              disabled={!editable || pending}
              onClick={() => {
                if (deleteTarget) remove.mutate(deleteTarget);
              }}
            >
              {remove.isPending ? "Deleting…" : "Delete rank"}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

function RankEditor({
  rank,
  editable,
  pending,
  error,
  onDirty,
  onSave,
  onCancel,
  onDelete,
}: {
  rank?: Rank;
  editable: boolean;
  pending: boolean;
  error: string;
  onDirty: (dirty: boolean) => void;
  onSave: (input: RankInput) => void;
  onCancel: () => void;
  onDelete?: () => void;
}) {
  const initial = {
    slug: rank?.slug ?? "",
    display_name: rank?.display_name ?? "",
    icon_url: rank?.icon_url ?? "",
    description: rank?.description ?? "",
  };
  const [draft, setDraft] = useState(initial);
  const dirty = Object.keys(initial).some(
    (key) => draft[key as keyof typeof draft] !== initial[key as keyof typeof initial],
  );
  const update = (key: keyof typeof draft, value: string) => {
    const next = { ...draft, [key]: value };
    setDraft(next);
    onDirty(
      Object.keys(initial).some(
        (field) => next[field as keyof typeof next] !== initial[field as keyof typeof initial],
      ),
    );
  };
  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!editable || pending) return;
    onSave({
      slug: draft.slug.trim(),
      display_name: draft.display_name.trim() || null,
      icon_url: draft.icon_url.trim() || null,
      description: draft.description.trim() || null,
    });
  };
  return (
    <form className="role-editor" onSubmit={submit} aria-busy={pending}>
      <header className="editor-heading">
        <div>
          <h2>{rank?.display_name ?? rank?.slug ?? "New rank"}</h2>
          <p>Ranks describe the unit’s organization.</p>
        </div>
        {!editable && (
          <span className="readonly-label">
            <LockKeyhole size={14} />
            Read only
          </span>
        )}
      </header>
      {!editable && (
        <p className="access-note">
          A role with Manage ranks permission is required to edit ranks.
        </p>
      )}
      <fieldset disabled={pending}>
        <Label htmlFor="rank-slug">Abbreviation</Label>
        <Input
          id="rank-slug"
          readOnly={!editable}
          value={draft.slug}
          required
          maxLength={100}
          onChange={(event) => update("slug", event.target.value)}
        />
        <Label htmlFor="rank-name">
          Rank name <span className="optional-label">optional</span>
        </Label>
        <Input
          id="rank-name"
          readOnly={!editable}
          value={draft.display_name}
          maxLength={100}
          onChange={(event) => update("display_name", event.target.value)}
        />
        <Label htmlFor="rank-icon">
          Icon URL <span className="optional-label">optional</span>
        </Label>
        <Input
          id="rank-icon"
          readOnly={!editable}
          type="url"
          value={draft.icon_url}
          maxLength={2000}
          pattern="https?://.*"
          onChange={(event) => update("icon_url", event.target.value)}
        />
        <Label htmlFor="rank-description">
          Description <span className="optional-label">optional</span>
        </Label>
        <Textarea
          id="rank-description"
          readOnly={!editable}
          value={draft.description}
          maxLength={2000}
          rows={4}
          onChange={(event) => update("description", event.target.value)}
        />
      </fieldset>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      {editable && (
        <footer className="editor-footer">
          <div className="editor-status">
            {onDelete && (
              <Button type="button" variant="outline" disabled={pending} onClick={onDelete}>
                <Trash2 size={15} />
                Delete rank
              </Button>
            )}
            <span>{dirty ? "Unsaved changes" : rank ? "Up to date" : ""}</span>
          </div>
          <div className="actions">
            <Button
              type="button"
              variant="ghost"
              disabled={pending || (!dirty && !!rank)}
              onClick={() => {
                setDraft(initial);
                onCancel();
              }}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={pending || (!dirty && !!rank) || !draft.slug.trim()}>
              Save rank
            </Button>
          </div>
        </footer>
      )}
    </form>
  );
}
