import { Users } from "lucide-react";
import type { ReactNode } from "react";

export function EmptyState({
  title,
  children,
  action,
}: {
  title: string;
  children: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className="state-panel">
      <Users size={26} aria-hidden="true" />
      <h2>{title}</h2>
      <p>{children}</p>
      {action}
    </div>
  );
}
