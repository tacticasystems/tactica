import type { Role } from "../lib/types";

import { FormError } from "./form-error";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";

export function DeleteRoleDialog({
  open,
  onOpenChange,
  role,
  pending,
  error,
  canDelete,
  onDelete,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  role: Role | null;
  pending: boolean;
  error: string;
  canDelete: boolean;
  onDelete: (role: Role) => void;
}) {
  return (
    <AlertDialog
      open={open}
      onOpenChange={(open) => {
        if (!pending) onOpenChange(open);
      }}
    >
      <AlertDialogContent
        className="delete-role-dialog"
        aria-busy={pending}
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          const target =
            document.querySelector<HTMLButtonElement>(".delete-role-button:not(:disabled)") ??
            document.querySelector<HTMLButtonElement>(".role-row.selected:not(:disabled)");

          target?.focus();
        }}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>Delete role?</AlertDialogTitle>
          <AlertDialogDescription>
            This permanently deletes “{role?.display_name}” and removes it from every member. This
            cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <FormError message={error} />
        <AlertDialogFooter>
          <AlertDialogCancel disabled={pending}>Cancel</AlertDialogCancel>
          <Button
            variant="destructive"
            disabled={pending || !role || !canDelete}
            onClick={() => {
              if (role) onDelete(role);
            }}
          >
            {pending ? "Deleting…" : "Delete role"}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
