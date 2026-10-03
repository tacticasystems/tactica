import type { ReactNode } from "react";

export function PageHeading({
  title,
  description,
  titleAdornment,
  action,
}: {
  title: string;
  description: string;
  titleAdornment?: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className="page-heading">
      <div>
        {titleAdornment ? (
          <div className="page-title">
            <h1>{title}</h1>
            {titleAdornment}
          </div>
        ) : (
          <h1>{title}</h1>
        )}
        <p>{description}</p>
      </div>
      {action}
    </div>
  );
}
