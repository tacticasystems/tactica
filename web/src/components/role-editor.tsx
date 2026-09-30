import { LockKeyhole, Trash2 } from "lucide-react";
import { useState } from "react";
import type { FormEvent } from "react";

import { canCreateRole, canEditRole, canGrant, permissionDefinitions } from "../lib/permissions";
import type { Access, Role, RoleInput } from "../lib/types";

import { FormError } from "./form-error";
import { Button } from "./ui/button";
import { Checkbox } from "./ui/checkbox";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Textarea } from "./ui/textarea";

export function RoleEditor({
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
                <Checkbox
                  checked={(permissions & bit) !== 0}
                  disabled={!editable || pending || !grantable}
                  onCheckedChange={(checked) =>
                    update(
                      name,
                      description,
                      checked === true ? permissions | bit : permissions & ~bit,
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
      <FormError message={error} />
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
