import { AlertCircle, ArrowRight, Users } from "lucide-react";
import { useState } from "react";
import type { ReactNode } from "react";
import { Link } from "@tanstack/react-router";
import { ApiError } from "../lib/session-client";
import { initials, safeImage } from "../lib/utils";
import { Button } from "./ui/button";

export function Avatar({
  name,
  url,
  size = "normal",
}: {
  name: string;
  url?: string | null;
  size?: "normal" | "small";
}) {
  const [failed, setFailed] = useState(false);
  const src = safeImage(url ?? null);
  return (
    <span className={`avatar avatar-${size}`} aria-hidden="true">
      {src && !failed ? <img src={src} alt="" onError={() => setFailed(true)} /> : initials(name)}
    </span>
  );
}

export function ErrorState({ error, retry }: { error: unknown; retry?: () => void }) {
  const denied = error instanceof ApiError && error.status === 403;
  const missing = error instanceof ApiError && error.status === 404;
  return (
    <div className="state-panel" role="alert">
      <AlertCircle size={24} aria-hidden="true" />
      <h2>
        {denied
          ? "This unit is for its members"
          : missing
            ? "This unit could not be found"
            : "Something went wrong"}
      </h2>
      <p>
        {denied
          ? "Choose a unit you belong to to view its personnel and roles."
          : error instanceof Error
            ? error.message
            : "Try again in a moment."}
      </p>
      <div className="actions">
        {retry && !denied && !missing && (
          <Button variant="outline" onClick={retry}>
            Try again
          </Button>
        )}
        <Button asChild variant="ghost">
          <Link to="/units">
            Choose a unit <ArrowRight size={16} />
          </Link>
        </Button>
      </div>
    </div>
  );
}

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

export function LoadingState({ label = "Loading workspace" }: { label?: string }) {
  return (
    <div className="loading-state" role="status" aria-label={label}>
      <span className="sr-only">{label}</span>
      <div className="skeleton skeleton-heading" />
      {Array.from({ length: 5 }, (_, index) => (
        <div className="skeleton skeleton-row" key={index} />
      ))}
    </div>
  );
}

export function PageHeading({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <div className="page-heading">
      <div>
        <h1>{title}</h1>
        <p>{description}</p>
      </div>
      {action}
    </div>
  );
}

export function Brand() {
  return (
    <>
      <img className="brand-logo" src="/tactica-logo.png" alt="" width={34} height={34} />
      <span>Tactica</span>
    </>
  );
}
