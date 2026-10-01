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
import { GripVertical, LockKeyhole, ChevronsUp } from "lucide-react";
import { useIsMobile } from "../hooks/use-mobile";
import { canManageRanks } from "../lib/permissions";
import { moveRank } from "../lib/rank-order";
import type { Access, Rank } from "../lib/types";
import { Button } from "./ui/button";

export function RankList({
  ranks,
  access,
  selected,
  pending,
  reorderStatus,
  reorderError,
  onSelect,
  onReorder,
}: {
  ranks: Rank[];
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
  const movable = canManageRanks(access) ? ranks : [];
  const canSort = movable.length > 1;
  const enabled = canSort && !pending && (!mobile || reorderMode);
  const sensors = useSensors(
    useSensor(MouseSensor, { activationConstraint: { distance: 5 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 200, tolerance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
  const rankName = (id: string | number) =>
    ranks.find((rank) => rank.id === id)?.display_name ??
    ranks.find((rank) => rank.id === id)?.slug ??
    "Rank";
  return (
    <section className="role-list" aria-label="Unit ranks">
      <div className="role-list-heading">
        <h2>Unit ranks</h2>
        <span>{ranks.length}</span>
        {mobile && canManageRanks(access) && (
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
              "Press Space to pick up a rank, use Up and Down arrows to move it, press Space to drop, or Escape to cancel.",
          },
          announcements: {
            onDragStart: ({ active }) => `${rankName(active.id)} picked up.`,
            onDragOver: ({ active, over }) =>
              over
                ? `${rankName(active.id)} moving to ${rankName(over.id)}'s position.`
                : undefined,
            onDragEnd: ({ active, over }) =>
              over && moveRank(access, ranks, String(active.id), String(over.id))
                ? `${rankName(active.id)} dropped. Saving rank order.`
                : "Rank order unchanged.",
            onDragCancel: () => "Reordering cancelled. Rank order unchanged.",
          },
        }}
        onDragStart={() => setDragging(true)}
        onDragCancel={() => setDragging(false)}
        onDragEnd={({ active, over }) => {
          setDragging(false);
          if (!enabled || !over) return;
          const order = moveRank(access, ranks, String(active.id), String(over.id));
          if (order) onReorder(order);
        }}
      >
        <SortableContext
          items={movable.map((rank) => rank.id)}
          strategy={verticalListSortingStrategy}
        >
          {ranks.map((rank) => (
            <SortableRank
              key={rank.id}
              rank={rank}
              selected={selected === rank.id}
              pending={pending || dragging}
              enabled={enabled && movable.some((item) => item.id === rank.id)}
              showHandle={(!mobile || reorderMode) && canManageRanks(access)}
              editable={canManageRanks(access)}
              onSelect={() => onSelect(rank.id)}
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
        Highest rank first.{" "}
        {canManageRanks(access) &&
          (mobile
            ? "Tap Reorder, then hold a handle and drag."
            : "Drag a handle to reorder ranks.")}
      </p>
    </section>
  );
}

function SortableRank({
  rank,
  selected,
  pending,
  enabled,
  showHandle,
  editable,
  onSelect,
}: {
  rank: Rank;
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
  } = useSortable({ id: rank.id, disabled: !enabled });
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
          aria-label={`Reorder “${rank.display_name ?? rank.slug}”`}
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
        <ChevronsUp size={17} />
        <span>
          <strong>{rank.display_name ?? rank.slug}</strong>
          <small>{rank.slug}</small>
        </span>
        {!editable && <LockKeyhole size={14} aria-label="Read only" />}
      </button>
    </div>
  );
}
