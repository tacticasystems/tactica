import { Skeleton } from "./ui/skeleton";

export function LoadingState({ label = "Loading workspace" }: { label?: string }) {
  return (
    <div className="loading-state" role="status" aria-label={label}>
      <span className="sr-only">{label}</span>
      <Skeleton className="skeleton skeleton-heading" />
      {Array.from({ length: 5 }, (_, index) => (
        <Skeleton className="skeleton skeleton-row" key={index} />
      ))}
    </div>
  );
}
