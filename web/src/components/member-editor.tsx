import { useMutation } from "@tanstack/react-query";
import { useBlocker, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { canAssignRanks, canAssignRole, canManageMemberProfiles } from "../lib/permissions";
import { queryClient } from "../lib/queries";
import { ApiError } from "../lib/session-client";
import type { Access, Member, MemberInput, Rank, Role } from "../lib/types";
import { Button } from "./ui/button";
import { Checkbox } from "./ui/checkbox";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { useWorkspace } from "./workspace-context";

export function MemberEditor({
  member,
  ranks,
  roles,
  access,
}: {
  member: Member;
  ranks: Rank[];
  roles: Role[];
  access: Access;
}) {
  const { unit, source, queryKey } = useWorkspace();
  const navigate = useNavigate();
  const [draftRank, setDraftRank] = useState<string | null>(null);
  const [draftName, setDraftName] = useState<string | undefined>();
  const [roleEdits, setRoleEdits] = useState<Record<string, boolean>>({});
  const rankId = draftRank ?? member.rank_id;
  const displayName = draftName ?? member.unit_display_name ?? "";
  const nameDirty = (displayName.trim() || null) !== (member.unit_display_name ?? null);
  const rankDirty = rankId !== member.rank_id;
  const explicitRoles = roles.filter((role) => role.kind !== "everyone");
  const selectedRoles = explicitRoles
    .filter((role) => roleEdits[role.id] ?? member.role_ids.includes(role.id))
    .map((role) => role.id);
  const changedRoles = explicitRoles.filter(
    (role) => selectedRoles.includes(role.id) !== member.role_ids.includes(role.id),
  );
  const dirty = nameDirty || rankDirty || changedRoles.length > 0;
  const canName = canManageMemberProfiles(access);
  const canRank = canAssignRanks(access);
  const editable = canName || canRank || explicitRoles.some((role) => canAssignRole(access, role));
  const permitted =
    (!nameDirty || canName) &&
    (!rankDirty || canRank) &&
    changedRoles.every((role) => canAssignRole(access, role));
  const returnToMember = () =>
    void navigate({
      to: "/units/$unitId/personnel/$memberId",
      params: { unitId: unit.id, memberId: member.id },
      ignoreBlocker: true,
    });
  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: [...queryKey, "members"] });
    void queryClient.invalidateQueries({ queryKey: [...queryKey, "member", member.id] });
    void queryClient.invalidateQueries({ queryKey: [...queryKey, "role-members"] });
    void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
  };
  const change = useMutation({
    mutationFn: async (input: MemberInput) => {
      await source.saveMember(unit.id, member.id, input);
      return source.member(unit.id, member.id);
    },
    onSuccess: async (saved) => {
      await queryClient.cancelQueries({ queryKey: [...queryKey, "members"] });
      await queryClient.cancelQueries({ queryKey: [...queryKey, "member", member.id] });
      queryClient.setQueriesData<Member[]>({ queryKey: [...queryKey, "members"] }, (members) =>
        members?.map((item) => (item.id === member.id ? saved : item)),
      );
      queryClient.setQueryData([...queryKey, "member", member.id], saved);
      setDraftRank(null);
      setDraftName(undefined);
      setRoleEdits({});
      refresh();
      returnToMember();
    },
    onError: () => {
      // A lost response may still have committed; refresh all affected views.
      refresh();
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "ranks"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] });
    },
  });
  const blocker = useBlocker({
    shouldBlockFn: () => dirty || change.isPending,
    enableBeforeUnload: dirty || change.isPending,
    withResolver: true,
  });
  const name = member.display_name ?? member.username;
  return (
    <>
      {blocker.status === "blocked" && (
        <div className="unsaved-prompt" role="alert">
          <p>
            {change.isPending
              ? "Your changes are being saved."
              : "You have unsaved changes. Discard them to leave this page?"}
          </p>
          <div className="actions">
            <Button
              type="button"
              variant="outline"
              disabled={change.isPending}
              onClick={() => {
                setDraftRank(null);
                setDraftName(undefined);
                setRoleEdits({});
                blocker.proceed();
              }}
            >
              Discard changes
            </Button>
            <Button type="button" variant="ghost" onClick={() => blocker.reset()}>
              Keep editing
            </Button>
          </div>
        </div>
      )}
      <form
        className="member-edit-form"
        aria-label={`Edit ${name}`}
        aria-busy={change.isPending}
        onSubmit={(event) => {
          event.preventDefault();
          if (!change.isPending && dirty && permitted && ranks.some((rank) => rank.id === rankId)) {
            const input: MemberInput = {};
            if (nameDirty) input.display_name = displayName.trim() || null;
            if (rankDirty) input.rank_id = rankId;
            if (changedRoles.length) input.role_ids = selectedRoles;
            change.mutate(input);
          }
        }}
      >
        <div className="rank-field">
          <Label htmlFor="member-display-name">Display name</Label>
          <Input
            id="member-display-name"
            value={displayName}
            onChange={(event) => setDraftName(event.target.value)}
            maxLength={100}
            placeholder={member.display_name ?? member.username}
            readOnly={!canName}
            disabled={change.isPending}
            aria-describedby="member-name-help"
          />
          <p className="member-field-help" id="member-name-help">
            {canName
              ? "Shown in this unit. Leave blank to use the account’s display name or username."
              : "Manage member profiles permission is required to change the display name."}
          </p>
        </div>
        <div className="rank-field">
          <Label htmlFor="member-rank">Rank</Label>
          <Select
            value={rankId}
            onValueChange={setDraftRank}
            disabled={!canRank || change.isPending}
          >
            <SelectTrigger id="member-rank">
              <SelectValue placeholder="Choose a rank" />
            </SelectTrigger>
            <SelectContent>
              {[...ranks]
                .sort((a, b) => b.position - a.position)
                .map((rank) => (
                  <SelectItem key={rank.id} value={rank.id}>
                    {rank.display_name ?? rank.slug}
                  </SelectItem>
                ))}
            </SelectContent>
          </Select>
          {!canRank && (
            <p className="member-field-help">
              Assign ranks permission is required to change the rank.
            </p>
          )}
        </div>
        <fieldset className="member-role-options" disabled={change.isPending}>
          <legend>Roles</legend>
          <p className="member-field-help">
            Everyone applies automatically. Roles at or above your highest role require unit
            ownership to change.
          </p>
          {explicitRoles.length === 0 ? (
            <p className="member-field-help">No assignable roles have been created.</p>
          ) : (
            explicitRoles.map((role) => (
              <label className="member-role-option" key={role.id}>
                <Checkbox
                  checked={selectedRoles.includes(role.id)}
                  disabled={change.isPending || !canAssignRole(access, role)}
                  onCheckedChange={(checked) =>
                    setRoleEdits((previous) => ({ ...previous, [role.id]: checked === true }))
                  }
                />
                <span>
                  <strong>{role.display_name}</strong>
                  {role.description && <small>{role.description}</small>}
                  {!canAssignRole(access, role) && <small>Read only</small>}
                </span>
              </label>
            ))
          )}
        </fieldset>
        {change.isError && (
          <p className="form-error" role="alert">
            {change.error instanceof ApiError && change.error.status === 403
              ? "Your permissions changed, or a role is above your highest role. Review the available fields and try again."
              : change.error instanceof Error
                ? change.error.message
                : "Changes could not be saved. Try again."}
          </p>
        )}
        <footer className="editor-footer">
          <span className="editor-status">{dirty ? "Unsaved changes" : "Up to date"}</span>
          <div className="actions">
            <Button
              type="button"
              variant="ghost"
              disabled={change.isPending}
              onClick={returnToMember}
            >
              Cancel
            </Button>
            {editable && (
              <Button
                type="submit"
                disabled={
                  change.isPending ||
                  !dirty ||
                  !permitted ||
                  !ranks.some((rank) => rank.id === rankId)
                }
              >
                {change.isPending ? "Saving…" : "Save changes"}
              </Button>
            )}
          </div>
        </footer>
      </form>
    </>
  );
}
