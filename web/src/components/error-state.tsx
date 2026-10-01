import { Link } from "@tanstack/react-router";
import { AlertCircle, ArrowRight } from "lucide-react";

import { ApiError } from "../lib/session-client";

import { Button } from "./ui/button";

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
