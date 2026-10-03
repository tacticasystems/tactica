import type { ReactNode } from "react";
import type { ResourceField } from "../lib/resource";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  TableCaption,
} from "./ui/table";

export function ResourceTable<T extends { id: string }, Context>({
  records,
  fields,
  context,
  caption,
  action,
}: {
  records: readonly T[];
  fields: readonly ResourceField<T, Context>[];
  context: Context;
  caption: string;
  action?: (record: T) => ReactNode;
}) {
  const columns = fields.filter((field) => field.in.includes("list"));
  return (
    <div className="table-wrap">
      <Table className="roster-table">
        <TableCaption className="sr-only">{caption}</TableCaption>
        <TableHeader>
          <TableRow>
            {columns.map((field) => (
              <TableHead key={field.id} scope="col">
                {field.label}
              </TableHead>
            ))}
            {action && (
              <TableHead scope="col">
                <span className="sr-only">Actions</span>
              </TableHead>
            )}
          </TableRow>
        </TableHeader>
        <TableBody>
          {records.map((record) => (
            <TableRow key={record.id}>
              {columns.map((field) => (
                <TableCell key={field.id}>{field.read(record, context)}</TableCell>
              ))}
              {action && <TableCell>{action(record)}</TableCell>}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
