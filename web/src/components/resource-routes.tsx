import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createRoute,
  Link,
  useBlocker,
  useNavigate,
  useParams,
  type AnyRoute,
} from "@tanstack/react-router";
import { Search } from "lucide-react";
import { useState } from "react";
import type { ResourceDefinition, ResourceState } from "../lib/resource";
import { ApiError } from "../lib/session-client";
import { EmptyState } from "./empty-state";
import { ErrorState } from "./error-state";
import { LoadingState } from "./loading-state";
import { PageHeading } from "./page-heading";
import { ResourceTable } from "./resource-table";
import { RosterPagination } from "./roster-pagination";
import { Button } from "./ui/button";
import { Input } from "./ui/input";

/** Sibling routes preserve existing URLs and parent layouts. Register only supported operations. */
export function createResourceRoutes<
  T extends { id: string },
  Context,
  Parent extends AnyRoute,
  const Path extends string,
  const Id extends string,
>(parent: Parent, path: Path, idParam: Id, resource: ResourceDefinition<T, Context>) {
  const key = resource.key ?? resource.apiBase.split("/").filter(Boolean).at(-1)!;
  const itemKey = resource.itemKey ?? `${key}-item`;
  const label = resource.label.toLowerCase();
  const plural = resource.pluralLabel.toLowerCase();

  function useResourceLocation() {
    const params = useParams({ strict: false }) as Record<string, string>;
    const base = `${parent.fullPath.replace(/\/$/, "")}/${path.replace(/^\//, "")}`.replace(
      /\$([^/]+)/g,
      (_match, name: string) => encodeURIComponent(params[name]),
    );
    const id = params[idParam];
    return { base, id, item: `${base}/${encodeURIComponent(id ?? "")}` };
  }

  function ListView() {
    const state = resource.useResource();
    const { base } = useResourceLocation();
    const [offset, setOffset] = useState(0);
    const [search, setSearch] = useState("");
    const records = useQuery({
      queryKey: [...state.queryKey, key, offset],
      queryFn: ({ signal }) => state.source.list(offset, signal),
      placeholderData: keepPreviousData,
    });
    const filtered = records.data?.filter(
      (record) =>
        !resource.searchText ||
        resource.searchText(record).toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()),
    );
    const retry = () => {
      void records.refetch();
      state.refetch?.();
    };
    return (
      <>
        <PageHeading
          title={resource.pluralLabel}
          description={state.description ?? `Browse ${plural}.`}
          action={
            resource.searchText && (
              <label className="search-input">
                <Search size={17} aria-hidden="true" />
                <span className="sr-only">Search this page</span>
                <Input
                  type="search"
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder={
                    state.total !== undefined && state.total <= 20
                      ? `Search ${plural}`
                      : "Search this page"
                  }
                />
              </label>
            )
          }
        />
        <div className="roster-meta">
          {state.total !== undefined && (
            <span>
              {state.total} {state.total === 1 ? label : plural}
            </span>
          )}
          {search && <span>{filtered?.length ?? 0} on this page match</span>}
          {records.isFetching && !records.isPending && <span role="status">Updating…</span>}
        </div>
        {records.isPending || state.pending ? (
          <LoadingState label={`Loading ${plural}`} />
        ) : records.isError || state.error ? (
          <ErrorState error={records.error ?? state.error} retry={retry} />
        ) : filtered?.length === 0 ? (
          <EmptyState
            title={search ? `No matching ${plural} on this page` : `No ${plural} on this page`}
          >
            {search
              ? "Try another name or clear your search."
              : "Choose the previous page if the list has changed."}
          </EmptyState>
        ) : (
          <ResourceTable
            records={filtered ?? []}
            fields={resource.fields}
            context={state.context}
            caption={state.caption ?? resource.pluralLabel}
            action={
              resource.listActions === false
                ? undefined
                : (record) => (
                    <Link to={String(`${base}/${encodeURIComponent(record.id)}`)}>
                      View {resource.name(record)}
                    </Link>
                  )
            }
          />
        )}
        {state.total !== undefined ? (
          (offset > 0 || state.total > 20) && (
            <RosterPagination
              offset={offset}
              count={records.data?.length ?? 0}
              total={state.total}
              pending={records.isFetching}
              onPageChange={(next) => {
                setSearch("");
                setOffset(next);
              }}
            />
          )
        ) : (
          <nav className="editor-footer" aria-label={`${resource.pluralLabel} pages`}>
            <Button
              variant="outline"
              disabled={!offset || records.isFetching}
              onClick={() => setOffset(Math.max(0, offset - 20))}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              disabled={records.isFetching || records.isError || (records.data?.length ?? 0) < 20}
              onClick={() => setOffset(offset + 20)}
            >
              Next
            </Button>
          </nav>
        )}
      </>
    );
  }

  function RecordView({ mode }: { mode: "view" | "edit" | "delete" }) {
    const state = resource.useResource();
    const location = useResourceLocation();
    const record = useQuery({
      queryKey: [...state.queryKey, itemKey, location.id],
      queryFn: ({ signal }) => state.source.get(location.id, signal),
    });
    const retry = () => {
      void record.refetch();
      state.refetch?.();
    };
    const back = (
      <Button asChild variant="ghost">
        <Link to={location.base}>Back to {resource.pluralLabel}</Link>
      </Button>
    );
    if (!record.data && record.error instanceof ApiError && record.error.status === 404)
      return (
        <>
          <PageHeading
            title={`${resource.label} not found`}
            description="This resource is no longer available."
            action={back}
          />
          <EmptyState title={`${resource.label} unavailable`}>
            Return to {resource.pluralLabel} to see the current list.
          </EmptyState>
        </>
      );
    if (record.isPending || state.pending) return <LoadingState label={`Loading ${label}`} />;
    if (!record.data || state.error)
      return <ErrorState error={record.error ?? state.error} retry={retry} />;
    const value = record.data;
    return (
      <>
        <PageHeading
          title={
            mode === "view"
              ? resource.name(value)
              : `${mode === "edit" ? "Edit" : "Delete"} ${label}`
          }
          description={state.description ?? resource.name(value)}
          action={back}
        />
        {record.isError && <ErrorState error={record.error} retry={retry} />}
        <section className="role-editor" aria-label={`${resource.label} details`}>
          {mode === "edit" ? (
            <Editor key={value.id} record={value} state={state} location={location} />
          ) : mode === "delete" ? (
            <Deletion record={value} state={state} location={location} />
          ) : (
            <>
              <dl className="member-details">
                {resource.fields
                  .filter((field) => field.in.includes("view"))
                  .map((field) => (
                    <div key={field.id}>
                      <dt>{field.label}</dt>
                      <dd>{field.read(value, state.context)}</dd>
                    </div>
                  ))}
              </dl>
              <footer className="editor-footer">
                {state.source.update && state.canEdit?.(value) && (
                  <Button asChild variant="outline">
                    <Link to={String(`${location.item}/edit`)}>Edit {label}</Link>
                  </Button>
                )}
                {state.source.remove && state.canDelete?.(value) && (
                  <Button asChild variant="destructive">
                    <Link to={String(`${location.item}/delete`)}>Delete {label}</Link>
                  </Button>
                )}
              </footer>
            </>
          )}
        </section>
      </>
    );
  }

  type RecordProps = {
    record: T;
    state: ResourceState<T, Context>;
    location: ReturnType<typeof useResourceLocation>;
  };
  function Editor({ record, state, location }: RecordProps) {
    const fields = resource.fields.filter((field) => field.in.includes("edit") && field.edit);
    const [values, setValues] = useState<Record<string, string>>(() =>
      Object.fromEntries(fields.map((field) => [field.id, field.edit!.value(record)])),
    );
    const dirty = fields.some((field) => values[field.id] !== field.edit!.value(record));
    const client = useQueryClient();
    const navigate = useNavigate();
    const save = useMutation({
      mutationFn: async () => {
        if (!state.canEdit?.(record) || !state.source.update)
          throw new Error("You cannot edit this resource.");
        const input: Partial<T> = Object.assign(
          {},
          ...fields
            .filter((field) => values[field.id] !== field.edit!.value(record))
            .map((field) => field.edit!.write(values[field.id])),
        );
        await state.source.update(record.id, input);
      },
      onSuccess: async () => {
        await client.invalidateQueries({ queryKey: state.queryKey });
        await navigate({ to: location.item, ignoreBlocker: true });
      },
      onError: () => {
        void client.invalidateQueries({ queryKey: state.queryKey });
      },
    });
    const blocker = useBlocker({
      shouldBlockFn: () => (dirty && !save.isSuccess) || save.isPending,
      enableBeforeUnload: dirty || save.isPending,
      withResolver: true,
    });
    if (!state.canEdit?.(record) || !state.source.update)
      return <EmptyState title="Editing unavailable">You cannot edit this resource.</EmptyState>;
    return (
      <>
        {blocker.status === "blocked" && (
          <div className="unsaved-prompt" role="alert">
            <p>
              {save.isPending
                ? "Your changes are being saved."
                : "You have unsaved changes. Discard them to leave this page?"}
            </p>
            <div className="actions">
              <Button variant="outline" disabled={save.isPending} onClick={() => blocker.proceed()}>
                Discard changes
              </Button>
              <Button variant="ghost" onClick={() => blocker.reset()}>
                Keep editing
              </Button>
            </div>
          </div>
        )}
        <form
          className="member-edit-form"
          aria-label={`Edit ${resource.name(record)}`}
          aria-busy={save.isPending}
          onSubmit={(event) => {
            event.preventDefault();
            if (dirty && !save.isPending) save.mutate();
          }}
        >
          {fields.map((field) => (
            <label className="rank-field" key={field.id}>
              <span>{field.label}</span>
              <Input
                value={values[field.id]}
                required={field.edit!.required}
                disabled={save.isPending}
                onChange={(event) => setValues({ ...values, [field.id]: event.target.value })}
              />
            </label>
          ))}
          {save.isError && (
            <p className="form-error" role="alert">
              {save.error.message}
            </p>
          )}
          <footer className="editor-footer">
            <Button type="submit" disabled={!dirty || save.isPending}>
              {save.isPending ? "Saving…" : "Save changes"}
            </Button>
            <Button asChild variant="ghost" disabled={save.isPending}>
              <Link to={location.item}>Cancel</Link>
            </Button>
          </footer>
        </form>
      </>
    );
  }

  function Deletion({ record, state, location }: RecordProps) {
    const client = useQueryClient();
    const navigate = useNavigate();
    const remove = useMutation({
      mutationFn: async () => {
        if (!state.canDelete?.(record) || !state.source.remove)
          throw new Error("You cannot delete this resource.");
        await state.source.remove(record.id);
      },
      onSuccess: async () => {
        await navigate({ to: location.base, ignoreBlocker: true });
        client.removeQueries({ queryKey: [...state.queryKey, itemKey, record.id], exact: true });
        await client.invalidateQueries({ queryKey: state.queryKey });
      },
      onError: () => {
        void client.invalidateQueries({ queryKey: state.queryKey });
      },
    });
    useBlocker({ shouldBlockFn: () => remove.isPending, enableBeforeUnload: remove.isPending });
    if (!state.canDelete?.(record) || !state.source.remove)
      return <EmptyState title="Deletion unavailable">You cannot delete this resource.</EmptyState>;
    return (
      <>
        <p>This permanently deletes “{resource.name(record)}”. This cannot be undone.</p>
        {remove.isError && (
          <p className="form-error" role="alert">
            {remove.error.message}
          </p>
        )}
        <footer className="editor-footer">
          <Button variant="destructive" disabled={remove.isPending} onClick={() => remove.mutate()}>
            {remove.isPending ? "Deleting…" : `Delete ${label}`}
          </Button>
          <Button asChild variant="outline" disabled={remove.isPending}>
            <Link to={location.item}>Cancel</Link>
          </Button>
        </footer>
      </>
    );
  }

  const list = createRoute({
    getParentRoute: () => parent,
    path,
    component: resource.listView ?? ListView,
  });
  const view = createRoute({
    getParentRoute: () => parent,
    path: `${path}/$${idParam}` as `${Path}/$${Id}`,
    component: resource.viewView ?? (() => <RecordView mode="view" />),
  });
  const edit = createRoute({
    getParentRoute: () => parent,
    path: `${path}/$${idParam}/edit` as `${Path}/$${Id}/edit`,
    component: resource.editView ?? (() => <RecordView mode="edit" />),
  });
  const remove = createRoute({
    getParentRoute: () => parent,
    path: `${path}/$${idParam}/delete` as `${Path}/$${Id}/delete`,
    component: resource.deleteView ?? (() => <RecordView mode="delete" />),
  });
  return { list, view, edit, remove, routes: [list, view, edit, remove] as const };
}
