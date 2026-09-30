import { useState } from "react";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical, LockKeyhole, Shield } from "lucide-react";
import { useIsMobile } from "../hooks/use-mobile";
import { canEditRole, canManageRoles } from "../lib/permissions";
import { moveRole } from "../lib/role-order";
import type { Access, Role } from "../lib/types";
import { Button } from "./ui/button";

export function RoleList({
  roles,
  access,
  selected,
  pending,
  reorderStatus,
  reorderError,
  onSelect,
  onReorder,
}: {
  roles: Role[];
  access: Access;
  selected?: string;
  pending: boolean;
  reorderStatus: string;
  reorderError: boolean;
  onSelect: (id: string) => void;
  onReorder: (order: string[]) => void;
}) {
  const mobile = useIsMobile();
  const [reorderMode, setReorderMode] = useState(false);
  const [dragging, setDragging] = useState(false);
  const movable = roles.filter((role) => role.kind === "custom" && canEditRole(access, role));
  const canSort = movable.length > 1;
  const enabled = canSort && !pending && (!mobile || reorderMode);
  const sensors = useSensors(
    useSensor(MouseSensor, { activationConstraint: { distance: 5 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 200, tolerance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
  const roleName = (id: string | number) =>
    roles.find((role) => role.id === id)?.display_name ?? "Role";
  return (
    <section className="role-list" aria-label="Unit roles">
      <div className="role-list-heading">
        <h2>Unit roles</h2>
        <span>{roles.length}</span>
        {mobile && canManageRoles(access) && (
          <Button
            variant="ghost"
            size="sm"
            disabled={pending || !canSort}
            onClick={() => setReorderMode(!reorderMode)}
            aria-pressed={reorderMode}
          >
            {reorderMode ? "Done" : "Reorder"}
          </Button>
        )}
      </div>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        accessibility={{
          screenReaderInstructions: {
            draggable:
              "Press Space to pick up a role, use Up and Down arrows to move it, press Space to drop, or Escape to cancel.",
          },
          announcements: {
            onDragStart: ({ active }) => `${roleName(active.id)} picked up.`,
            onDragOver: ({ active, over }) =>
              over
                ? `${roleName(active.id)} moving to ${roleName(over.id)}'s position.`
                : undefined,
            onDragEnd: ({ active, over }) =>
              over && moveRole(access, roles, String(active.id), String(over.id))
                ? `${roleName(active.id)} dropped. Saving role order.`
                : "Role order unchanged.",
            onDragCancel: () => "Reordering cancelled. Role order unchanged.",
          },
        }}
        onDragStart={() => setDragging(true)}
        onDragCancel={() => setDragging(false)}
        onDragEnd={({ active, over }) => {
          setDragging(false);
          if (!enabled || !over) return;
          const order = moveRole(access, roles, String(active.id), String(over.id));
          if (order) onReorder(order);
        }}
      >
        <SortableContext
          items={movable.map((role) => role.id)}
          strategy={verticalListSortingStrategy}
        >
          {roles.map((role) => (
            <SortableRole
              key={role.id}
              role={role}
              selected={selected === role.id}
              pending={pending || dragging}
              enabled={enabled && movable.some((item) => item.id === role.id)}
              showHandle={
                (!mobile || reorderMode) && role.kind === "custom" && canEditRole(access, role)
              }
              editable={canEditRole(access, role)}
              onSelect={() => onSelect(role.id)}
            />
          ))}
        </SortableContext>
      </DndContext>
      <div
        className={`role-order-feedback ${reorderError ? "form-error" : ""}`}
        role={reorderError ? "alert" : "status"}
      >
        {reorderStatus}
      </div>
      <p className="role-order-note">
        Highest role first.{" "}
        {mobile
          ? "Tap Reorder, then hold a handle and drag."
          : "Drag a handle to reorder custom roles."}{" "}
        Administrator and Everyone stay fixed.
      </p>
    </section>
  );
}

function SortableRole({
  role,
  selected,
  pending,
  enabled,
  showHandle,
  editable,
  onSelect,
}: {
  role: Role;
  selected: boolean;
  pending: boolean;
  enabled: boolean;
  showHandle: boolean;
  editable: boolean;
  onSelect: () => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: role.id, disabled: !enabled });
  return (
    <div
      ref={setNodeRef}
      className={`role-list-item ${selected ? "selected" : ""} ${isDragging ? "dragging" : ""}`}
      style={{
        transform: CSS.Transform.toString(transform ? { ...transform, x: 0 } : null),
        transition,
      }}
    >
      {showHandle && (
        <button
          type="button"
          className="role-drag-handle"
          ref={setActivatorNodeRef}
          {...attributes}
          {...listeners}
          disabled={!enabled}
          aria-label={`Reorder “${role.display_name}”`}
          title="Drag to reorder; Space and arrow keys also work"
        >
          <GripVertical size={16} aria-hidden="true" />
        </button>
      )}
      <button
        className={`role-row ${selected ? "selected" : ""}`}
        type="button"
        onClick={onSelect}
        aria-pressed={selected}
        disabled={pending}
      >
        <Shield size={17} />
        <span>
          <strong>{role.display_name}</strong>
          <small>
            {role.kind === "everyone"
              ? "Applies to every member"
              : role.kind === "administrator"
                ? "Built-in administrator"
                : "Custom role"}
          </small>
        </span>
        {!editable && <LockKeyhole size={14} aria-label="Read only" />}
      </button>
    </div>
  );
}
