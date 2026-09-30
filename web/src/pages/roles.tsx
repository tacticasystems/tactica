import { Textarea } from "../components/ui/textarea";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { useEffect, useState } from "react";
import type { FormEvent, ReactNode } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useBlocker, useNavigate, useSearch } from "@tanstack/react-router";
import { Check, LockKeyhole, Plus, Trash2, X } from "lucide-react";
import { useWorkspace } from "../components/workspace";
import { EmptyState, ErrorState, LoadingState, PageHeading } from "../components/shared";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../components/ui/alert-dialog";
import { Button } from "../components/ui/button";
import {
  canCreateRole,
  canDeleteRole,
  canEditRole,
  canGrant,
  permissionDefinitions,
} from "../lib/permissions";
import { RoleList } from "../components/role-list";
import { RoleMembers } from "../components/role-members";
import { errorMessage, queryClient } from "../lib/queries";
import type { Access, Role, RoleInput } from "../lib/types";

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
  const reorder = useMutation({
    mutationFn: (roleIds: string[]) => source.reorderRoles(unit.id, roleIds),
    onMutate: async (roleIds) => {
      await queryClient.cancelQueries({ queryKey: [...queryKey, "roles"] });
      const previous = queryClient.getQueryData<Role[]>([...queryKey, "roles"]);
      const byId = new Map(previous?.map((item) => [item.id, item]));
      queryClient.setQueryData<Role[]>(
        [...queryKey, "roles"],
        roleIds.map((id, index) => ({ ...byId.get(id)!, position: roleIds.length - index - 1 })),
      );
      return { previous };
    },
    onSuccess: async (updated) => {
      queryClient.setQueryData([...queryKey, "roles"], updated);
      await queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
    onError: (_error, _order, context) => {
      if (context?.previous) queryClient.setQueryData([...queryKey, "roles"], context.previous);
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });
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
          <button
            type="button"
            aria-label="Dismiss saved message"
            className="icon-button"
            onClick={() => setNotice("")}
          >
            <X size={15} />
          </button>
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
      <AlertDialog
        open={deleteOpen}
        onOpenChange={(open) => {
          if (!remove.isPending) setDeleteOpen(open);
        }}
      >
        <AlertDialogContent
          className="delete-role-dialog"
          aria-busy={remove.isPending}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            document
              .querySelector<HTMLButtonElement>(
                ".delete-role-button:not(:disabled), .role-row.selected:not(:disabled)",
              )
              ?.focus();
          }}
        >
          <AlertDialogHeader>
            <AlertDialogTitle>Delete role?</AlertDialogTitle>
            <AlertDialogDescription>
              This permanently deletes “{deleteTarget?.display_name}” and removes it from every
              member. This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          {remove.isError && (
            <p role="alert" className="form-error">
              {errorMessage(remove.error)}
            </p>
          )}
          <AlertDialogFooter>
            <AlertDialogCancel disabled={remove.isPending}>Cancel</AlertDialogCancel>
            <Button
              variant="destructive"
              disabled={
                remove.isPending || !deleteTarget || !canDeleteRole(capabilities, deleteTarget)
              }
              onClick={() => {
                if (deleteTarget) remove.mutate(deleteTarget);
              }}
            >
              {remove.isPending ? "Deleting…" : "Delete role"}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

function RoleDetails({
  role,
  access,
  pending,
  children,
}: {
  role?: Role;
  access: Access;
  pending: boolean;
  children: ReactNode;
}) {
  const [membersOpen, setMembersOpen] = useState(false);
  const showMembers = !!role && membersOpen;
  return (
    <div className="role-details">
      {role && (
        <nav className="role-view-switch" aria-label="Role details">
          <Button
            type="button"
            variant={membersOpen ? "ghost" : "secondary"}
            aria-pressed={!membersOpen}
            onClick={() => setMembersOpen(false)}
          >
            Permissions
          </Button>
          <Button
            type="button"
            variant={membersOpen ? "secondary" : "ghost"}
            aria-pressed={membersOpen}
            onClick={() => setMembersOpen(true)}
          >
            Members
          </Button>
        </nav>
      )}
      <div hidden={showMembers}>{children}</div>
      {role && showMembers && (
        <RoleMembers key={role.id} role={role} access={access} pending={pending} />
      )}
    </div>
  );
}

function RoleEditor({
  role,
  access,
  pending,
  error,
  onSave,
  onCancel,
  onDirty,
  onDelete,
}: {
  role?: Role;
  access: Access;
  pending: boolean;
  error: string;
  onDelete?: () => void;
  onSave: (input: RoleInput) => void;
  onCancel: () => void;
  onDirty: (dirty: boolean) => void;
}) {
  const [name, setName] = useState(role?.display_name ?? "");
  const [description, setDescription] = useState(role?.description ?? "");
  const [permissions, setPermissions] = useState(role?.permissions ?? 0);
  const everyone = role?.kind === "everyone";
  const editable = role ? canEditRole(access, role) : canCreateRole(access);
  const dirty =
    name !== (role?.display_name ?? "") ||
    description !== (role?.description ?? "") ||
    permissions !== (role?.permissions ?? 0);
  const update = (nextName: string, nextDescription: string, nextPermissions: number) => {
    setName(nextName);
    setDescription(nextDescription);
    setPermissions(nextPermissions);
    onDirty(
      nextName !== (role?.display_name ?? "") ||
        nextDescription !== (role?.description ?? "") ||
        nextPermissions !== (role?.permissions ?? 0),
    );
  };
  const reset = () => {
    update(role?.display_name ?? "", role?.description ?? "", role?.permissions ?? 0);
    onCancel();
  };
  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!editable || pending) return;
    onSave(
      everyone
        ? { permissions }
        : { display_name: name.trim(), description: description.trim() || null, permissions },
    );
  };
  return (
    <form className="role-editor" onSubmit={submit} aria-busy={pending}>
      <header className="editor-heading">
        <div>
          <h2>{role ? role.display_name : "New role"}</h2>
          <p>
            {role
              ? "Review and edit this role’s access."
              : "Give the role a name and choose its permissions."}
          </p>
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
          {role?.kind === "administrator"
            ? "The built-in Administrator role is protected and cannot be edited."
            : "Your permissions or role position do not allow you to edit this role."}
        </p>
      )}
      {everyone && (
        <p className="access-note">
          Everyone applies to all current and future members. Only its permissions can be edited.
        </p>
      )}
      <fieldset disabled={!editable || pending}>
        <Label htmlFor="role-name">Role name</Label>
        <Input
          id="role-name"
          value={name}
          onChange={(event) => update(event.target.value, description, permissions)}
          required
          maxLength={100}
          readOnly={everyone}
        />
        <Label htmlFor="role-description">
          Description <span className="optional-label">optional</span>
        </Label>
        <Textarea
          id="role-description"
          value={description}
          onChange={(event) => update(name, event.target.value, permissions)}
          maxLength={2000}
          rows={3}
          readOnly={everyone}
        />
        <div className="permissions-heading">
          <h3>Permissions</h3>
          <p>Ranks do not grant permissions.</p>
        </div>
        <div className="permissions-list">
          {permissionDefinitions.map(({ bit, label, description: help, future }) => {
            const grantable = canGrant(access, bit) || ((role?.permissions ?? 0) & bit) !== 0;
            return (
              <Label className="permission-row" key={bit}>
                <Input
                  type="checkbox"
                  checked={(permissions & bit) !== 0}
                  disabled={!grantable}
                  onChange={(event) =>
                    update(
                      name,
                      description,
                      event.target.checked ? permissions | bit : permissions & ~bit,
                    )
                  }
                />
                <span>
                  <strong>
                    {label}
                    {future && <small className="planned-label">Planned</small>}
                  </strong>
                  <small>
                    {help}
                    {!grantable && " You cannot grant this permission."}
                  </small>
                </span>
              </Label>
            );
          })}
        </div>
      </fieldset>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <footer className="editor-footer">
        <div className="editor-status">
          {onDelete && (
            <Button
              type="button"
              variant="outline"
              className="delete-role-button"
              disabled={pending}
              onClick={onDelete}
            >
              <Trash2 size={15} />
              Delete role
            </Button>
          )}
          <span>{dirty ? "Unsaved changes" : role ? "Up to date" : ""}</span>
        </div>
        <div className="actions">
          <Button
            type="button"
            variant="ghost"
            onClick={reset}
            disabled={pending || (!dirty && !!role)}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={!editable || pending || (!dirty && !!role) || !name.trim()}
          >
            {pending ? "Saving…" : role ? "Save changes" : "Create role"}
          </Button>
        </div>
      </footer>
    </form>
  );
}
