import { useMutation, useQuery } from "@tanstack/react-query";
import { useBlocker, useNavigate, useSearch } from "@tanstack/react-router";
import { Check, Plus, X } from "lucide-react";
import { useEffect, useState } from "react";

import { useRoleReorder } from "../hooks/use-role-reorder";
import { canCreateRole, canDeleteRole } from "../lib/permissions";
import { errorMessage, queryClient } from "../lib/queries";
import type { Role, RoleInput } from "../lib/types";

import { DeleteRoleDialog } from "../components/delete-role-dialog";
import { EmptyState } from "../components/empty-state";
import { ErrorState } from "../components/error-state";
import { LoadingState } from "../components/loading-state";
import { PageHeading } from "../components/page-heading";
import { RoleDetails } from "../components/role-details";
import { RoleEditor } from "../components/role-editor";
import { RoleList } from "../components/role-list";
import { Button } from "../components/ui/button";
import { useWorkspace } from "../components/workspace-context";

export function RolesPage() {
  const { unit, source, queryKey, preview } = useWorkspace();
  const { roleId } = useSearch({ from: "/units/$unitId/roles" });
  const navigate = useNavigate({ from: "/units/$unitId/roles" });
  const selected = roleId ?? null;

  const setSelected = (id: string | null) => {
    // Callers either guard the switch or explicitly save/discard the draft.
    void navigate({ search: id ? { roleId: id } : {}, ignoreBlocker: true });
  };

  const roles = useQuery({
    queryKey: [...queryKey, "roles"],
    queryFn: ({ signal }) => source.roles(unit.id, signal),
  });

  const access = useQuery({
    queryKey: [...queryKey, "access"],
    queryFn: ({ signal }) => source.access(unit.id, signal),
  });

  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [notice, setNotice] = useState("");
  const [dirty, setDirty] = useState(false);
  const [switchTarget, setSwitchTarget] = useState<string | null>(null);

  const defaultId = roles.data?.find((role) => role.kind === "custom")?.id ?? roles.data?.[0]?.id;
  const validSelection = selected === "new" || roles.data?.some((role) => role.id === selected);

  const currentId = validSelection ? (selected ?? undefined) : defaultId;

  useEffect(() => {
    if (roles.isSuccess && selected && !validSelection) {
      void navigate({ search: {}, replace: true, ignoreBlocker: true });
    }
  }, [roles.isSuccess, selected, validSelection, navigate]);

  const blocker = useBlocker({
    shouldBlockFn: ({ current, next }) =>
      dirty &&
      (current.pathname !== next.pathname ||
        ("roleId" in next.search ? next.search.roleId : defaultId) !== currentId),
    enableBeforeUnload: dirty,
    withResolver: true,
  });

  const role = roles.data?.find((role) => role.id === currentId);

  const select = (id: string) => {
    if (id === currentId) return;
    if (dirty) {
      setSwitchTarget(id);
      return;
    }
    setSelected(id);
    setNotice("");
  };

  const save = useMutation({
    mutationFn: ({ roleId, input }: { roleId: string | null; input: RoleInput }) =>
      source.saveRole(unit.id, roleId, input),
    onSuccess: async (saved) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] }),
        queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] }),
      ]);
      setDirty(false);
      setSelected(saved.id);
      setNotice("Role saved.");
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });

  const remove = useMutation({
    mutationFn: (target: Role) => source.deleteRole(unit.id, target.id),
    onSuccess: async (_result, target) => {
      queryClient.setQueryData<Role[]>([...queryKey, "roles"], (existing) =>
        existing?.filter((item) => item.id !== target.id),
      );
      setDeleteOpen(false);
      setSelected(null);
      setDirty(false);
      setSwitchTarget(null);
      save.reset();
      setNotice(`“${target.display_name}” deleted.`);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] }),
        queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] }),
        queryClient.invalidateQueries({ queryKey: [...queryKey, "members"] }),
      ]);
    },
    onError: () => {
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "members"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });

  const reorder = useRoleReorder();

  const pending = save.isPending || remove.isPending || reorder.isPending;
  if (roles.isPending || access.isPending)
    return (
      <>
        <PageHeading title="Roles" description="Control access to your unit workspace." />
        <LoadingState label="Loading roles and permissions" />
      </>
    );

  if (roles.isError || access.isError)
    return (
      <ErrorState
        error={roles.error ?? access.error}
        retry={() => {
          void roles.refetch();
          void access.refetch();
        }}
      />
    );

  const capabilities = access.data;

  return (
    <>
      <PageHeading
        title="Roles"
        description="Roles grant permissions. Ranks describe your organization."
        action={
          canCreateRole(capabilities) && (
            <Button variant="outline" onClick={() => select("new")} disabled={pending}>
              <Plus size={16} />
              New role
            </Button>
          )
        }
      />
      {preview && (
        <p className="roles-preview-note">
          Roles and assignments are preview content for trying the UI. They do not describe access
          in 9 Rifles.
        </p>
      )}
      {notice && (
        <div className="save-notice" role="status">
          <Check size={16} />
          {notice}
          <Button
            variant="ghost"
            size="icon"
            type="button"
            aria-label="Dismiss saved message"
            className="icon-button"
            onClick={() => setNotice("")}
          >
            <X size={15} />
          </Button>
        </div>
      )}
      {(switchTarget || blocker.status === "blocked") && (
        <div className="unsaved-prompt" role="alert">
          <p>
            {blocker.status === "blocked" && blocker.next.pathname !== blocker.current.pathname
              ? "You have unsaved changes. Discard them to leave this page?"
              : "You have unsaved changes. Discard them to open another role?"}
          </p>
          <div className="actions">
            <Button
              variant="outline"
              onClick={() => {
                if (blocker.status === "blocked") blocker.proceed();
                else setSelected(switchTarget);
                setSwitchTarget(null);
                setDirty(false);
                save.reset();
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
        <RoleList
          roles={roles.data}
          access={capabilities}
          selected={currentId}
          pending={pending}
          reorderStatus={
            reorder.isPending
              ? "Saving order…"
              : reorder.isError
                ? "Order wasn’t saved. Try again."
                : reorder.isSuccess
                  ? "Order saved."
                  : ""
          }
          reorderError={reorder.isError}
          onSelect={(id) => {
            save.reset();
            select(id);
          }}
          onReorder={(order) => {
            setSelected(currentId ?? null);
            reorder.mutate(order);
          }}
        />
        {currentId === "new" || role ? (
          <RoleDetails role={role} access={capabilities} pending={pending}>
            <RoleEditor
              key={currentId}
              role={role}
              access={capabilities}
              pending={pending}
              error={save.isError ? errorMessage(save.error) : ""}
              onDelete={
                role && canDeleteRole(capabilities, role)
                  ? () => {
                      remove.reset();
                      setDeleteTarget(role);
                      setDeleteOpen(true);
                    }
                  : undefined
              }
              onDirty={setDirty}
              onSave={(input) => {
                setNotice("");
                save.mutate({ roleId: role?.id ?? null, input });
              }}
              onCancel={() => {
                save.reset();
                setDirty(false);
                setSelected(
                  roles.data.find((item) => item.kind === "custom")?.id ??
                    roles.data[0]?.id ??
                    null,
                );
              }}
            />
          </RoleDetails>
        ) : (
          <EmptyState title="No roles to display">
            Create a role to define access for your unit.
          </EmptyState>
        )}
      </div>
      <DeleteRoleDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        role={deleteTarget}
        pending={remove.isPending}
        error={remove.isError ? errorMessage(remove.error) : ""}
        canDelete={!!deleteTarget && canDeleteRole(capabilities, deleteTarget)}
        onDelete={(target) => remove.mutate(target)}
      />
    </>
  );
}
